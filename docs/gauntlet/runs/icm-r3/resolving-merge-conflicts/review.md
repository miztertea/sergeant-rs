# Independent adversarial review: resolving-merge-conflicts (ICM-R3)

Reviewer position per `reference/proposal-icm-r-procedure-authority.md`
§8.11 and `docs/icm/record-shapes.md` §6 rule 1: a fresh execution, review-only
contract, no edit authority over the live package or the producer's draft
(`docs/gauntlet/runs/icm-r3/resolving-merge-conflicts/adjudication-draft.md`).
Re-derivation was performed directly against
`.sergeant/workflows/resolving-merge-conflicts/` (CONTEXT.md, index.md,
workflow.toml, both stage `CONTEXT.md`/`output/README.md` files), the
upstream source (`reference/sergeant-upstream/.agents/skills/
resolving-merge-conflicts/SKILL.md`), and the archived provenance
(`docs/gauntlet/promoted-provenance/resolving-merge-conflicts.md`) — not
from the producer's own citations, per this Work's brief.

Challenge checklist applied per §8.11: source fidelity, rung order (PL and
J), Captain/workflow boundary, stage/helper boundary, authority grants and
missing J0 cases, package identity/naming, duplicated or drift-prone
content, false pairing assumptions, unjustified engine gaps.

## Per-unit verdicts

### BU-P3-045 — verdict: CONFIRMED

Independent re-derivation: `SKILL.md` frontmatter description plus the
5-step body together establish "resolve an in-progress git merge/rebase
conflict to completion without inventing behavior or aborting" as the
package's bounded outcome; `CONTEXT.md` and `index.md` restate this
faithfully. PL-4 holds — this is a durably executable procedure from an
already-conflicted state to a terminal result, matching §5.6's discriminator
list ("merge-conflict resolution" is explicitly named as an example likely
to fit PL-4). No PL-0/1/2/3 candidate exists: nothing in `sgt`'s current
surfaces owns hunk-level intent-preserving resolution, and the driver is a
stage actor acting on an already-existing conflict, not Captain deciding
whether one should be resolved (correctly distinguished from the PL-2
discriminator). The producer's J5 citation ("never invent behavior; never
abort," restated at every stage) is a legitimate governing constraint,
independently verifiable in both stage `CONTEXT.md` files' Behavior
contract sections. No source-fidelity, rung-order, or boundary defect found.

### BU-P3-046 — verdict: CONFIRMED

Independent re-derivation against §5.8 (PL-6 test: "repeatable machinery
whose output follows mechanically from declared inputs and whose invocation
does not itself require substantive judgment"): inspecting git history and
listing conflicting files is mechanical discovery with no branching
decision — it produces the same output for the same repository state
regardless of actor. The fold into `10-research-intent` as a subordinate
helper (rather than an independent stage) passes the reimplementation test
(§5.7): no operator would need to independently retry, measure, or gate
this checkpoint in isolation from the research it feeds. Agrees with the
producer.

### BU-P3-047 — verdict: CONFIRMED

Independent re-derivation confirms the missing-J0 finding. Checking the
ladder in order against `10-research-intent/CONTEXT.md`'s actual text: J5 —
no governing constraint in the stage or workflow content speaks to "no
primary source can be found"; J4 — no explicit user/Work decision is
recorded; J3 — no settled record exists (this is a fresh conflict, not a
previously adjudicated one); J2 — the stage's Behavior contract delegates
*which* sources to trace ("commit messages, PRs, and issues/tickets") but
never states what to do when none exist. The stage's only escalation
language is the generic "ask the user where the behavior contract above
requires it" boilerplate in "## Judgment required," which is not a named J0
trigger in the required shape (`convention.md` §6.1). J0 is correctly
reached. The analogy to `research`'s own drafted J0 clause ("No primary
source can be found for a claim the requester needs answered") is apt: both
are source-tracing steps with structurally identical failure modes.

### BU-P3-048 — verdict: CONFIRMED

Independent re-derivation confirms both the PL-5 rung and the missing-J0
finding for the tie-break case. J5 governs "never invent, never abort" but
is silent on *which side wins* when neither preservation nor a discoverable
"stated goal" resolves the conflict; J4/J3 do not apply (no standing
user decision or settled record addresses an unknown future conflict's
specific tie); J2 delegates picking the side matching "the merge's stated
goal," which presupposes that a goal is discoverable — it does not delegate
authority to invent one when it is not. Reaching J0 for the genuinely
irreconcilable case is correct and the recommended clause is well-formed
(states evidence checked, offers a recommendation, asks one direct
question, per the canonical J0 shape in the Bounded-Judgment Ladder §6.7).

### BU-P3-049 — verdict: NEEDS-REVISION

The PL-6 classification and the fold into `20-resolve-hunks` are confirmed:
"discover and run typecheck/tests/format" is genuinely mechanical, and
`30-validate`'s original demotion holds under re-derivation. But the
producer's authority analysis for this unit is incomplete. The stage
contract text is "fixing anything the merge broke" — the producer assigns
this only a "J2 sliver ... bounded by J5," with no J0 case considered. Two
concrete failure modes are unaddressed:

1. **Check failures that are not merge-induced.** If typecheck/tests/format
   fail for a reason predating the merge (a pre-existing flaky test, an
   unrelated lint violation), "fixing anything the merge broke" does not
   delegate authority to modify code the merge did not touch — yet the
   contract gives the actor no way to distinguish "broke because of this
   merge" from "was already broken," and no escalation path when it cannot
   tell. This is not covered by BU-P3-048's governing constraint (which
   only forbids invented *behavior*, not unrelated fixes), so it is a
   genuine J2/J0 boundary the producer's table does not name.
2. **A fix that itself requires a design decision.** "Fixing" a broken
   check can range from a one-line adjustment to a change that trades off
   behavior (e.g., a type error whose correct fix depends on which side's
   intent should govern — the same class of judgment BU-P3-048 already
   requires escalation for at the hunk level, but here recurring one layer
   later, after hunks are already resolved).

Both are missing-J0-case findings under §8.11's checklist ("authority
grants and missing J0 cases") that the producer's adjudication did not
surface, even though it correctly surfaced two structurally similar gaps
elsewhere in the same package. Recommended clause: when an automated check
fails and the actor cannot establish that the failure is attributable to
this merge, or the correct fix itself requires a judgment call beyond
mechanical repair (i.e., it is not obviously implied by the hunk resolution
already recorded), the actor records what was checked and asks the user
rather than guessing at the correct fix. This should be added to the
`## Bounded judgment` section this package's remediation already plans to
write for `20-resolve-hunks`.

### BU-P3-050 — verdict: NEEDS-REVISION

The PL-6 classification for "stage everything and commit; if rebasing,
continue until every commit is rebased" is only partly sound. Staging and
committing already-resolved changes is mechanical. But "continue the rebase
process until all commits are rebased" is not guaranteed to be mechanical:
`git rebase --continue` commonly re-applies the next commit in the series,
which can surface a *new* conflict requiring the same judgment
`20-resolve-hunks`'s own Behavior contract (BU-P3-048) already governs.
Neither the upstream `SKILL.md` step 5 nor the current `20-resolve-hunks/
CONTEXT.md` Helper section states what happens when continuing the rebase
re-enters a conflicted state — the fold's own text presents "finish" as a
single mechanical closing action, with no explicit loop-back reference to
the hunk-resolution behavior it may need to re-invoke per subsequent commit.
This is a stage/helper-boundary concern under §8.11: folding BU-P3-050 into
`20-resolve-hunks` as pure PL-6 machinery implicitly assumes rebase
continuation cannot re-trigger the very judgment this fold is subordinate
to, and that assumption is not stated or defended anywhere in the package.
This does not necessarily require re-opening the A4 fold or promoting a new
stage (the reviewing single stage already owns both the judgment and the
mechanical close, so a re-entered conflict can be handled by looping back
inside the same checkpoint without a new execution boundary) — but the
package's Helper section should say so explicitly, rather than leaving the
multi-commit-rebase case as an undocumented gap. Recommend adding one
sentence to the `20-resolve-hunks` Helper section: "If continuing the
rebase surfaces a new conflict, treat it as a return to this stage's own
hunk-resolution behavior (BU-P3-048), not a fresh unaddressed state."

### BU-RMC-01 — verdict: CONFIRMED

Independently verified: `.sergeant/workflows/resolving-merge-conflicts/
CONTEXT.md` contains no `## Authority envelope` heading (grep-checked
directly against the current file, not the producer's characterization).
ADR 0013 decision 4 requires this section on every workflow-level
`CONTEXT.md` always. Gap is real.

### BU-RMC-02 — verdict: CONFIRMED

Independently verified: both `10-research-intent/CONTEXT.md` and
`20-resolve-hunks/CONTEXT.md` carry an identical "## Judgment required"
paragraph (compared byte-for-byte; the two are the same boilerplate text
with no stage-specific J2/J1/J0 content), not the required `## Bounded
judgment` shape with named J2 delegations, J1 local choices, and J0
triggers. This duplicated boilerplate is itself the drift-prone pattern the
ICM-R2 pilot review already flagged elsewhere in this same Work's brief.
Confirmed as both a compliance gap and a duplication concern.

## Additional finding not in the producer's table

**Ladder-vocabulary drift in `index.md`'s stage table (minor, does not
change any BU verdict).** `index.md`'s "Stages" table still cites each
stage's rung as "actor-stage (§6.4, judgment)" — the pre-ICM-R3 convention
ladder's section numbering, not the new PL-N vocabulary (`PL-5`) this
adjudication and the proposal's §5 ladder now use. The producer's "Surviving
package design" remediation list (Bounded-judgment sections, Authority
envelope section) does not mention updating this table. Recommend folding a
third item into the same remediation pass: update `index.md`'s ladder-rung
column to cite `PL-5`/`PL-6` per this adjudication, so the package's own
authored content does not cite a superseded ladder section number after the
other two amendments land.

## Overall verdict on Final disposition

**STAND is confirmed** — independently re-derived, not merely accepted from
the producer's citations. No PL-0/1/2/3 rehoming candidate exists, no stage
boundary is wrong, no false pairing with `research` was found (the shared
technique is real but the durable outcomes differ, matching proposal §8.8's
clustering test), and no engine gap survives the lower rungs. The package's
identity, stage list, and PL rungs all hold under direct re-derivation
against the current file content and the upstream source (source fidelity
confirmed: `docs/gauntlet/promoted-provenance/resolving-merge-conflicts.md`
matches `reference/sergeant-upstream/.agents/skills/
resolving-merge-conflicts/SKILL.md` exactly, byte-for-byte for the five
steps).

However, the producer's remediation list under "Surviving package design"
is **incomplete**, not merely correct-as-far-as-it-goes. Two additional
items belong in the same in-place-amendment pass before this package can be
called authority-valid:

1. A third J0 case in `20-resolve-hunks`'s eventual `## Bounded judgment`
   section, for automated-check failures that cannot be attributed to the
   merge or whose correct fix requires a judgment call beyond mechanical
   repair (BU-P3-049 above).
2. An explicit loop-back sentence addressing a rebase continuation that
   re-surfaces a conflict, so the PL-6 fold of `40-finish` does not
   silently assume rebase continuation is always conflict-free
   (BU-P3-050 above).

Neither finding overturns STAND, changes any PL rung, or reopens the A4
folds themselves (both folds are re-confirmed under independent
re-derivation) — they extend the same in-place content amendment the
producer already scoped, so they carry no REHOME/SPLIT/HARVEST
consequence under ADR 0013 decision 6. The producer's remediation plan
should be treated as a partial list, not a complete one, when this record
proceeds to reconcile-and-publish (§8.12).
