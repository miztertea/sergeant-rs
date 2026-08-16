# Package adjudication: to-tickets

ICM-R3 full-reconciliation pass, `reference/proposal-icm-r-procedure-
authority.md` §10.4; method per §8; record shape per
`docs/icm/record-shapes.md` §6; owner rulings per
`docs/adr/0013-icm-r0-owner-rulings.md`. Producer pass only — independent
review is a separate step (§8.11 of the proposal; `docs/icm/convention.md`
§6.2/6.3) and has not run yet. This record is itself draft and does not
self-promote (ADR 0013 decisions 6-7). Wave 2 of this reconciliation pass;
`00-load-project-context` delegates to **load-project** (wave 1), whose own
adjudication draft and independent review were read in full before this
pass began (`docs/gauntlet/runs/icm-r3/load-project/adjudication-draft.md`,
`review.md`) — see "Cross-package dependency on load-project" below.

## Original intention

Break a plan, spec, investigation, findings register, PR, or conversation
into dependency-aware tracer-bullet work: load project context, extract
only genuinely blocking unknowns as short investigation tickets, confirm
granularity/ownership/blocking-edge correctness with the user (unless
immediate publication was requested) and publish, then report the dispatch
frontier without authorizing dispatch (`.sergeant/workflows/to-tickets/
CONTEXT.md` "Purpose"/"Stages"; `index.md`). Promoted candidate **W32**
from the N1 manual reference-corpus decomposition
(`docs/gauntlet/contracts/N1.md`), decomposed from
`reference/sergeant-upstream`'s `to-tickets` skill
(`reference-corpus/synthesis.md` §1). Full N1 citation trail archived at
`docs/gauntlet/promoted-provenance/to-tickets.md`. This ICM-R3 pass does not
re-run N1 extraction; it re-derives placement and authority classification
against the current package content and checks it against the archived
provenance, the current upstream source, and a separate, later-dispositioned
citation batch (`docs/icm/agents-invariant-dispositions.md`, `skill:
to-tickets` rows) that promotion never landed in the package text — see
"The ticket-quality-rules gap" below.

## Current trigger and outcome

Four ordinary actor stages (`workflow.toml`: `00-load-project-context`,
`10-extract-decisions-and-unknowns`, `20-confirm-breakdown`,
`40-report-frontier` — `30-publish` demoted and folded into
`20-confirm-breakdown` at N1 adjudication A4, no renumbering), `status:
published`, `version: 2`, listed in `.sergeant/index.md` and routed by
`AGENTS.md` line 50 ("Substantive procedural work has a matching published
workflow" → the workflow's own `index.md`), consistent with `AGENTS.md`
line 233's own list of currently published workflows naming `to-tickets`
directly. Trigger (per `CONTEXT.md`, `index.md`, and every stage's own
Layer-2 contract, restated identically): "The user says 'to tickets',
'create issues', 'create td tasks', 'make epics', or asks to break
something into work."

Outcome: project context is loaded; an investigation ticket exists only for
a genuinely blocking unknown, each naming its exact deliverable; the
proposed breakdown is confirmed for granularity/ownership/blocking edges
(unless immediate publication was requested) and published, staying open,
with cross-repo blockers recorded as counterpart ids plus merge order; the
dispatch frontier is reported with a sensible default concurrency, without
that report itself authorizing dispatch.

Unlike this pass's wave-1 sibling `load-project`, `to-tickets` is not
dead in practice — it is directly named in `AGENTS.md`'s own routing
doctrine and its own catalog row.

## Driver and admission boundary

Driver: **stage actor**, all four stages (each already labeled
"actor-stage (§6.4, judgment)" in the package's own stage table — verified,
not merely copied, against the Placement Ladder below).

Admission boundary: **post-Work, in-Work**. The execution-surface test
(`convention.md` §2a — "would a human type `sgt run '<intent>' --workflow
to-tickets`?") holds: a human has already supplied the artifact to break
down (a plan, spec, investigation, findings register, PR, or conversation)
before this package's first stage runs; no stage's contract asks the actor
to decide *whether* that artifact should become tickets, only *how*. The
package does ask bounded questions (`BU-P4-068`'s confirmation gate), but
per §5.6 "a workflow may ask a bounded question during execution, but
conversation cannot be its primary product" — the primary product here is
published tickets and a frontier report, not dialogue. This confirms
PL-4/PL-5, not PL-2.

Known consumers/delegations, verified by direct search (not assumed from
the package's own text): no other live package under `.sergeant/workflows/`
or `skills/` names `to-tickets` (`grep -rn "to-tickets" .sergeant/workflows
skills AGENTS.md docs/DEVELOPMENT.md docs/icm`, cross-checked). `to-tickets`
itself delegates outward once, to **load-project**, addressed below.

## Cross-package dependency on load-project (wave 1)

`00-load-project-context/CONTEXT.md`'s own "## Delegation" section states:
"This stage's outcome is produced by running **load-project** to its own
completion (context composition today...)." Per this Work's brief, that
citation's accuracy was checked against `load-project`'s now-current
classification, not merely assumed carried-over from N1 promotion (which
only verified `load-project` was `status: published` at the time,
`promoted-provenance/to-tickets.md`'s own "NEEDS-JUDGMENT resolution"
note).

**Finding: the citation is stale relative to load-project's current
(draft, unreviewed) classification, but not yet incorrect against the live
filesystem.** `docs/gauntlet/runs/icm-r3/load-project/adjudication-draft.md`
independently re-derives that the multi-project registry `load-project`
wraps does not exist in the current product (`sergeant.toml`'s
`[estate]`/`[[repo]]`/`[group.<name>]` schema has no "project name" to
register or look up) and recommends **ABSORBED** — `load-project`'s
protective and context-resolution intents are already owned by
`estate-navigation` and `sgt` itself, at a stronger rung. That record's own
"Cross-package consequence" section already names this exact dependency and
already commits to the correct remedy: "when `load-project` is actually
retired at the reconcile-and-publish step, `to-tickets/00-load-project-
context/CONTEXT.md`'s 'Delegation' section must be corrected in the same
change to name `estate-navigation`... instead." Its independent review
(`docs/gauntlet/runs/icm-r3/load-project/review.md`) confirms the underlying
ABSORBED conclusion (disputing only table bookkeeping, not the verdict) and
does not disturb that commitment.

Today, `load-project` is still `status: published` under
`.sergeant/workflows/load-project/` and still listed in `.sergeant/
index.md` — its retirement has not been reconciled or published (ADR 0013
decisions 6-7: a producer's own draft does not self-promote). The
delegation therefore still resolves to a real, currently-admitted package;
it is not a broken `@@name`-class reference today. This pass does **not**
edit the delegation target itself: `load-project`'s own record already
claims that correction as belonging to its retirement's reconcile-and-
publish step (a coordinated edit across both packages' files), and
`to-tickets/**` is out of this producer pass's own write scope regardless
(per this Work's brief). This is recorded here as a required in-place
amendment, timed to land together with `load-project`'s retirement, not
before it and not by a different pass acting alone.

**Rungs checked (bounded-judgment.md order), for whether this producer may
correct the Delegation target now:** J5 — no constraint forbids waiting;
`docs/icm/convention.md` §4 rule 1's "broken reference" violation applies
only once the referent is actually gone, which is not yet true. J4 — the
task brief instructs verifying accuracy, not executing a cross-package
edit. J3 — `load-project`'s own adjudication draft is not yet an accepted
record (unreviewed); a draft does not settle J3 (`bounded-judgment.md` §J3:
"a draft, self-authored output... does not qualify"). J2 — this pass's
delegated judgment is to assess and record, not to pre-empt a two-file
coordinated edit gated on another package's own review. **Conclusion:
record the finding, do not edit now** — landing it early would create the
exact "identical in both trees / one boundary crossed twice" hazard
`convention.md` §2 rule 2 warns against for drafts, applied by analogy to a
same-change coordinated retirement.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| `BU-P4-058` | `CONTEXT.md`/`index.md` (Purpose, workflow-level) — turning a plan/spec/investigation/findings-register/PR/conversation into implementation-ready tracked tickets is a distinct, triggerable procedure | PL-4 | J5 (contract-level: package identity and stage order are fixed by `workflow.toml`, §1 rule 4) | STAND | `to-tickets` (workflow) |
| `BU-P4-064` | `00-load-project-context/CONTEXT.md` — do not automatically add td instructions to a repo's own guidance files as a side effect | PL-5 | J5 (governing: scope may not be silently widened by writing to a repository's own guidance files without being asked — matches the independently-landed `AGENTS.md` invariant `BU-1311`, same rule, different citation lane; consistent, not duplicative content since one lives in `AGENTS.md` and one is this stage's own local restatement of the same constraint as it applies to td-instruction files specifically) | STAND | `00-load-project-context` |
| n/a (delegation target) | `00-load-project-context/CONTEXT.md` "## Delegation" — names **load-project** | N/A (cross-package reference, not itself a behavior unit) | J2 checked, concludes "record now, edit later" — see "Cross-package dependency on load-project" above | STAND today; in-place amendment required, timed to `load-project`'s own reconcile-and-publish | `00-load-project-context/CONTEXT.md` (retarget to `estate-navigation` once `load-project` retires) |
| `BU-P4-065` | `10-extract-decisions-and-unknowns/CONTEXT.md` — create a short investigation ticket only when a genuinely blocking unknown cannot be answered from existing evidence, naming the exact deliverable | PL-5 | J2 (delegated: judge whether an unknown is genuinely blocking versus answerable from existing evidence) | STAND | `10-extract-decisions-and-unknowns` |
| `BU-P4-068` | `20-confirm-breakdown/CONTEXT.md` — unless immediate publication was requested, present the breakdown and ask only whether granularity/ownership/blocking edges are correct | PL-5 | J4 (the user's own explicit "publish immediately" request, if given, governs whether to ask at all) + J2 (delegated: how to present the breakdown for confirmation) | STAND | `20-confirm-breakdown` |
| `BU-P4-070` | `20-confirm-breakdown/CONTEXT.md` "Helper invocation: publish" (folded `30-publish`, N1 A4) — do not mark newly published tasks in_progress | PL-6 (helper, folded per A4) | J5 (governing, unconditional: ticket status must accurately reflect that planning, not execution, has occurred) | STAND | `20-confirm-breakdown` (helper invocation) |
| `BU-P4-071` | `20-confirm-breakdown/CONTEXT.md` "Helper invocation: publish" — record cross-repo blockers as counterpart repo/ticket id plus merge order, not a fabricated native dependency edge | PL-6 (helper, folded per A4) | J5 (governing, unconditional: never represent a cross-database dependency td cannot actually enforce) | STAND | `20-confirm-breakdown` (helper invocation) |
| `BU-P4-072` | `40-report-frontier/CONTEXT.md` — recommend one worker per owning repository as the default concurrency, unless the project explicitly supports more | PL-5 | J2 (delegated: what counts as "the project explicitly supports more") | STAND | `40-report-frontier` |
| `BU-P4-073` | `40-report-frontier/CONTEXT.md` — do not dispatch unless the user asked to begin implementation; reporting is not authorization | PL-5 | J5 (governing: matches proposal invariant 4.4, "execution is not dialogue" / no silent-trigger rule — reporting must never itself cause action) | STAND | `40-report-frontier` |
| `BU-1297`/`1298`/`1299`/`1301`/`1302`/`1303`/`1304`/`1305` (ticket-quality rules: vertical slices, one-fresh-context sizing, one owning repo, expand-migrate-contract for mechanical changes, epics vs. ticket substitutes, no duplicate tracker entries, preserve stable finding ids, readiness = observable acceptance criteria + accurate blockers) | `docs/icm/agents-invariant-dispositions.md` lines 197-205, dispositioned `skill: to-tickets` (not `AGENTS.md`) | PL-5 (stage-context; the dispositioning pass's own rationale — "belong to that workflow" — already names the destination rung, this pass confirms it) | J2 (same delegated judgment as `BU-P4-068`: what makes a proposed breakdown's granularity/ownership genuinely correct) | **FOLD — missing from the live package; not a placement change, a content gap** | `20-confirm-breakdown/CONTEXT.md` (add to Behavior contract) |
| `BU-1300` (counterpart tickets + explicit merge order, not one ambiguous shared cross-repo ticket) | `docs/icm/agents-invariant-dispositions.md` line 200, dispositioned `skill: to-tickets` | PL-6 (same helper as `BU-P4-071`) | J5 (same governing constraint) | STAND — already substantively covered by `BU-P4-071`'s live text (counterpart id + merge order), no new sentence required, citation added for completeness | `20-confirm-breakdown` (helper invocation) — citation-only addition |
| `BU-1311` (do not automatically add task-tracker instructions to repository guidance files) | `docs/icm/agents-invariant-dispositions.md` line 206, dispositioned `AGENTS.md` (not `skill: to-tickets`) | PL-1 (already correctly placed at `AGENTS.md`, per that pass's own disposition) | N/A (governs `AGENTS.md`, not this package) | n/a to this package — cited here only to confirm `BU-P4-064` above is not a duplicate of it, per §5.10's caution against inventing a second home for the same rule | `AGENTS.md` (unchanged, out of this pass's scope) |
| n/a (authoring-format compliance) | all four stage `CONTEXT.md` files — uniform `## Judgment required` boilerplate; no stage names J2 decision classes, J1 local choices, or J0 escalation triggers in the ADR 0013 shape | N/A | J5 (`convention.md` §6.1 + ADR 0013 decision 4: every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section "always... omission is never ambiguous" — a governing requirement this package predates) | STAND (package identity correct; in-place amendment required) | all four stage `CONTEXT.md` files |
| n/a | `CONTEXT.md` (L1) — no `## Authority envelope` section exists | N/A | J5 (`convention.md` §6.1: every workflow Layer-1 `CONTEXT.md` carries an `## Authority envelope` section) | STAND, in-place amendment required | `CONTEXT.md` |
| n/a | `20-confirm-breakdown/CONTEXT.md` "Helper invocation: publish" — "No `kind = \"execute\"` stage exists in the current engine, so the acting harness performs the publish operation itself" | N/A (factual-accuracy defect, not a placement question) | N/A | STAND, false claim requires correction | `20-confirm-breakdown/CONTEXT.md` |
| n/a | `CONTEXT.md` line 37 — "See `provenance.md` for the complete stage-to-behavior-unit mapping"; no `provenance.md` exists under `.sergeant/workflows/to-tickets/` | N/A (dangling reference, but systemic, not package-specific) | J1 (local, cosmetic) | STAND, no action required by this pass | `CONTEXT.md` (catalog-wide, not this package) |

## The ticket-quality-rules gap (BU-1297/1298/1299/1301/1302/1303/1304/1305, full record)

`docs/icm/agents-invariant-dispositions.md` (a later, separate MVP-5 Lane F1
pass reviewing candidate `AGENTS.md` invariants) explicitly dispositions
nine rows — `BU-1297` through `BU-1305` — as `skill: to-tickets` rather than
`AGENTS.md`, with the stated rationale "Ticket-sizing, ownership, cross-repo
counterpart, and epic rules belong to that workflow (published WORKFLOW per
retriage)." That same document's own "What this pass did not do" section is
explicit that this was "a placement judgment, not a content change to those
workflows" and that "library re-homing execution... is separate... content-
lane work" — i.e., the pass that decided *where* these nine units belong
never itself delivered them there.

Independent re-read of the live package (Inventory, §8.3) confirms none of
the eight substantive units (`BU-1297`, `1298`, `1299`, `1301`, `1302`,
`1303`, `1304`, `1305`) appear anywhere in `20-confirm-breakdown/CONTEXT.md`
or any other stage's Behavior contract. `20-confirm-breakdown` currently
states only that granularity, ownership, and blocking edges must be
*confirmed* (`BU-P4-068`) — it never states what correct granularity,
correct ownership, or a ready ticket actually look like, which is exactly
what the missing nine units supply: vertical-slice sizing (`BU-1297`,
`1298`), one owning repository per ticket (`BU-1299`), expand-migrate-
contract for mechanical non-vertical-slice changes (`BU-1301`), epics as
programs rather than ticket substitutes (`BU-1302`), no duplicate tracker
entries (`BU-1303`), preserved stable finding ids in ticket titles
(`BU-1304`), and observable-acceptance-criteria/accurate-blockers readiness
(`BU-1305`). The ninth, `BU-1300` (counterpart tickets plus explicit merge
order instead of one ambiguous shared cross-repo ticket), is already
substantively present in the live `BU-P4-071` text and does not need new
prose — only its own citation, for completeness of the corpus's own
citation trail.

This is not a placement error — `docs/icm/agents-invariant-dispositions.md`
correctly named `20-confirm-breakdown`'s home rung (PL-5, `skill:
to-tickets`) — it is the same **promotion/drafting gap** class this
reconciliation round already found in `deepen-module` (`BU-P4-018/019/025`,
`docs/gauntlet/runs/icm-r3/deepen-module/adjudication-draft.md`): real,
already-cited, already-classified content that never actually landed in the
admitted package text, discovered here because this pass read the full
corpus (Inventory, §8.3) rather than trusting the package's own citation
list as complete.

**Rungs checked, for whether this producer may fix the gap directly
(bounded-judgment.md order):**

- **J5** — No governing constraint forbids adding previously-cited,
  already-classified behavior-unit content into the stage it was already
  assigned to. Nothing here changes scope, public behavior, or authority —
  it completes content the corpus already classified.
- **J4** — No user/Work decision is in tension with adding it; the task
  brief instructs applying §8's method to every file under this package,
  which includes reconciling cited-but-undelivered content.
- **J3** — `docs/icm/agents-invariant-dispositions.md`'s own placement
  judgment ("belong to that workflow") is an accepted, already-landed
  document (not a draft of this pass), qualifying as a settled
  authoritative record under J3.
- **J2** — This reconciliation method's own Step 4/5/8 (Normalize,
  Placement classification, Draft) explicitly delegates completing a
  package's cited-but-undelivered content back into its already-classified
  destination.

**Conclusion: J2/J3, not J0.** Restoring the eight substantive units into
`20-confirm-breakdown/CONTEXT.md`'s Behavior contract, plus citing
`BU-1300` alongside the existing `BU-P4-071` text, is in-place content
completion, not a placement or disposition change. This producer marks it
as required remediation below rather than silently leaving the gap, or
worse, treating `to-tickets`'s Final disposition as fully authority-valid
when it is not.

## The false execute-stage claim in the publish helper invocation

`20-confirm-breakdown/CONTEXT.md`'s "## Helper invocation: publish" section
states: "No `kind = \"execute\"` stage exists in the current engine, so the
acting harness performs the publish operation itself." This claim is false
as of this branch, in the identical way `research/00-investigate/
CONTEXT.md` (also reconciled this round) already had this same claim
corrected: `.sergeant/workflows/repo-to-icm/workflow.toml`'s
`65-self-check` is a live `kind = "execute"` stage. Whether "publish to td,
with in_progress and cross-repo-blocker discipline" should become a
mechanical execute-stage check rather than trusted to this stage's own
judgment is a real open question this pass raises but does not resolve —
parking it as a follow-on finding (not built here; adding it would be new
`workflow.toml`/execute-stage content beyond this reconciliation pass's own
scope), not silently re-asserted as settled, matching the correction
pattern and its own stated boundary in `research/00-investigate/
CONTEXT.md`.

## Surviving package design

No stage moves, merges, splits, or renames; PL-4 (package) / PL-5 (each
stage, PL-6 for the folded publish helper) is confirmed, not merely
inherited from the package's own table. Disposition is **STAND**, requiring
in-place content amendment, not restructuring:

1. Add `BU-1297`, `1298`, `1299`, `1301`, `1302`, `1303`, `1304`, `1305`
   (ticket-quality rules) to `20-confirm-breakdown/CONTEXT.md`'s Behavior
   contract, citing `docs/icm/agents-invariant-dispositions.md` lines
   197-205 exactly as the rest of that stage already cites
   `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md`. Add a
   citation for `BU-1300` alongside the existing `BU-P4-071` text (no new
   prose needed).
2. Correct the false "no `kind = \"execute\"` stage exists" claim in
   `20-confirm-breakdown/CONTEXT.md`'s "Helper invocation: publish"
   section, in the same shape as `research/00-investigate/CONTEXT.md`'s own
   already-corrected rung-rationale note, and park (do not resolve) the
   execute-stage-for-publish question as a follow-on finding.
3. Replace each of the four stages' `## Judgment required` boilerplate with
   a `## Bounded judgment` section per `convention.md` §7.3 /
   `.sergeant/common/contexts/bounded-judgment.md`, naming this record's J2
   delegations, J4/J5 governing constraints, and (see below) the candidate
   J0 triggers this pass surfaces.
4. Add a `## Authority envelope` section to `CONTEXT.md` (L1) per
   `convention.md` §7.2.
5. When `load-project`'s own retirement lands at its reconcile-and-publish
   step (not this pass, not before that step), retarget
   `00-load-project-context/CONTEXT.md`'s "## Delegation" section from
   `load-project` to `estate-navigation`, matching `load-project`'s own
   adjudication record's committed remedy.
6. Leave the catalog-wide `provenance.md` reference as-is; consistent with
   the same non-local reference already found and left alone in
   `deepen-module`'s pass — correcting it catalog-wide is out of this
   single-package pass's scope.

Two candidate J0 cases surface for the stage-level `## Bounded judgment`
sections amendment 3 above must add, recorded here (not drafted as final
clause text — that is amendment 3's job, per the same discipline
`deepen-module`'s pass applied):

- `10-extract-decisions-and-unknowns`: no cited content states what happens
  when the evidence for "genuinely blocking" is itself contested — e.g. one
  reading of the source material treats an unknown as blocking, another
  treats it as safely deferrable. Checked: J5 no constraint requires or
  forbids resolving this unilaterally; J4 no user/Work decision
  pre-authorizes either reading; J3 no settled record addresses it; J2 the
  stage delegates judging whether an unknown blocks, not resolving a
  genuine disagreement about the evidence itself; J1 does not apply
  (creating an unnecessary investigation ticket, or omitting a necessary
  one, both have downstream cost). **Conclusion: J0** — "the evidence for
  whether an unknown is blocking is itself ambiguous or contested" is a
  candidate `needs_input` trigger.
- `20-confirm-breakdown`: `BU-1299` (exactly one owning repository per
  ticket) and cross-repo counterpart practice (`BU-P4-071`/`BU-1300`) can
  conflict when a single piece of work is not cleanly separable by
  repository — the behavior contract states the target shape but not what
  to do when a candidate ticket genuinely resists single-repo ownership.
  Checked: J5 no constraint forbids splitting further; J4/J3 no settled
  decision addresses this specific shape; J2 the stage delegates
  granularity/ownership judgment generally, not resolving an
  irreducibly-cross-repo unit; J1 does not apply (the choice affects
  downstream dispatch and merge order, not a locally reversible detail).
  **Conclusion: J0** — "a candidate ticket cannot be cleanly assigned a
  single owning repository" is a candidate `needs_input` trigger.

## Inputs and outputs

Inputs: all four stages' Inputs tables were read and verified against
`record-shapes.md` §1a — `00-load-project-context` correctly declares only
`../CONTEXT.md` (L1, first stage only); `10-extract-decisions-and-unknowns`,
`20-confirm-breakdown`, and `40-report-frontier` each correctly declare
their immediate predecessor's `output/README.md` (L4), and
`40-report-frontier`'s Inputs table correctly notes it now points at
`20-confirm-breakdown` (which absorbed the demoted `30-publish`) rather than
a stage that no longer exists. No undeclared contract-bearing dependency
found; no violation of §1a rule 1. Directory listing order (`00`, `10`,
`20`, `40`) agrees with `workflow.toml`'s declared stage order
(`docs/icm/convention.md` §1 rule 4) — verified directly.

Outputs: `00-load-project-context`, `10-extract-decisions-and-unknowns`, and
`20-confirm-breakdown` all declare `evidence` (Work-branch record only);
`40-report-frontier` declares `promote` (workflow deliverable), correctly
reflecting that it is the terminal stage.
`docs/gauntlet/promoted-provenance/to-tickets.md`'s own "Promotion note"
already records that this `promote`-with-no-finalize-step shape is
accepted, human-reviewed disposition (one of 30 of 34 N1 packages in that
shape) — this pass does not reopen that.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (`index.md`) — its structural and provenance
identity does not change. Remediation items 1-4 and 6 above are ordinary
content edits to an admitted workflow and go through this repository's
normal review path for workflow content changes, not a new
draft-and-promote cycle, per `docs/icm/convention.md` §2 (the
draft/admitted split governs *new or substantially rewritten* content;
adding previously-cited content and required sections to an already-
admitted stage's `CONTEXT.md` is neither). Item 5 (the `load-project`
delegation retarget) is explicitly gated on a separate package's own
reconcile-and-publish step and must not land ahead of it. Per ADR 0013
decision 6, only the promotable form of these changes (once actually made)
needs independent review before landing; this adjudication record itself
needs ICM-R3's own reviewer step (`reference/proposal-icm-r-procedure-
authority.md` §8.11) before its findings are treated as settled.

## Alternatives considered

- **Treat the missing `BU-1297`-`1305` content as HARVEST into a new shared
  `.sergeant/common/contexts/ticket-quality.md`.** Rejected: no second
  workflow consumer of these exact ticket-sizing/ownership/readiness rules
  was found (`grep` across `.sergeant/workflows/` for "vertical slice",
  "expand-migrate-contract", and "fresh agent context" surfaces no other
  package); `docs/icm/agents-invariant-dispositions.md`'s own placement
  judgment already names a single owning workflow. Revisit only if a second
  package is later found to need the identical rules verbatim.
- **Correct the `load-project` Delegation target now, on this producer's
  own authority, rather than waiting for that package's reconcile-and-
  publish step.** Rejected: `load-project`'s own adjudication is still an
  unreviewed producer draft (`review.md` recommends ABSORBED but flags the
  draft NEEDS-REVISION); retargeting `to-tickets` first would assert a
  retirement that has not been accepted, and would create exactly the
  "boundary crossed twice, once early" hazard `convention.md` §2 rule 2
  warns against for the draft/admitted split, applied here by analogy to a
  cross-package coordinated edit.
- **Resolve the two new J0 cases (contested-blocking evidence; irreducibly
  cross-repo tickets) on this producer's own authority**, drafting the
  actual `needs_input` trigger text now rather than only naming the gap.
  Rejected, same reasoning `deepen-module`'s pass used: this pass's job is
  producing the adjudication record, not landing the content amendments;
  inventing the clause wording without a reviewer pass first would collapse
  the self-check/independent-review separation this ladder exists to
  preserve (`convention.md` §6.2/6.3).
- **Leave `BU-1297`-`1305` undispositioned** on the theory that the
  package's `Final disposition` is STAND regardless, so the gap doesn't
  change the verdict. Rejected: §8.6/§8.9 require every behavior unit
  dispositioned before a package is ready to publish, and a citation that
  resolves to nothing in the live text is exactly the "package cannot be
  called authority-valid" failure mode `reference/proposal-icm-r-procedure-
  authority.md` §9.1 warns against — silence here would misrepresent this
  package as more complete than it is.
- **REHOME or SPLIT the package** on the theory that four separate content
  gaps (ticket-quality rules, a stale delegation, a false execute-stage
  claim, missing Bounded-judgment/Authority-envelope sections) signal a
  structurally wrong package. Rejected: every gap is an in-place content or
  citation defect inside stages whose PL-4/PL-5 placement is independently
  confirmed correct; none requires moving behavior to a different owning
  surface (`record-shapes.md` §6 rule 4 / proposal §8.8's file-shape-
  mirroring caution cuts the other way here — restructuring a correctly-
  placed package to explain a content gap would itself be the mirroring
  failure).

## Final disposition
STAND

## Validation evidence

- Source-valid: every citation currently in the live package was traced to
  `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md` and
  matches verbatim against `docs/gauntlet/promoted-provenance/
  to-tickets.md`. Additionally, nine cited-but-undelivered units
  (`BU-1297`-`1305`) were found by cross-checking the live package against
  `docs/icm/agents-invariant-dispositions.md`'s own `skill: to-tickets`
  disposition rows, not assumed complete from the package's own citation
  list — the same discipline applied to `deepen-module` this round.
- Placement-valid: every stage's already-recorded PL-5 rung (PL-6 for the
  folded publish helper) and the package's own PL-4 rung were independently
  re-derived from the Placement Ladder
  (`reference/proposal-icm-r-procedure-authority.md` §5) in this pass and
  confirmed, including a specific check of the PL-2/PL-4 discriminator
  against the package's own confirmation-gate prose (`BU-P4-068`) — see
  "Driver and admission boundary" above.
- Authority-valid: **not yet** — this pass found the same class of gap
  ICM-R2's `validate-and-ship` pass and this round's `deepen-module` pass
  found (no `## Bounded judgment` or `## Authority envelope` sections in
  the ADR 0013 shape), plus two new J0 cases surfaced while drafting the
  remediation this pass recommends, plus a false factual claim about the
  current engine's `kind = "execute"` support, plus a stale (not yet
  broken, but committed-to-become-broken) cross-package delegation. The
  package cannot be called authority-valid until remediation items 1-5
  under "Surviving package design" land, item 5 gated on `load-project`'s
  own retirement.
- Structurally valid: all four stage directories, their `output/README.md`
  declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly; directory
  listing order matches declared order despite the `30` gap left by N1
  adjudication A4's fold.
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the package;
  `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims remain to be measured separately.
- Wave-2 cross-package check (per this Work's brief): `00-load-project-
  context` delegates to **load-project** (wave 1); the citation was
  verified against `load-project`'s now-current classification (draft
  ABSORBED, unreviewed) rather than trusted as still meaning "an ordinary
  live workflow peer" — see "Cross-package dependency on load-project"
  above for the full finding and why this pass records rather than acts on
  it.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7); it does
  not self-promote.
