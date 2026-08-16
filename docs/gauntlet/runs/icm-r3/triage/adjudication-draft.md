# Package adjudication: triage

ICM-R3 full-reconciliation pass, `reference/proposal-icm-r-procedure-authority.md`
§10.4; method per §8; record shape per `docs/icm/record-shapes.md` §6.
Producer pass only — independent review is a separate step (§8.11 of the
proposal; §6.2/6.3 of `docs/icm/convention.md`) and has not run yet. This
record is itself draft and does not self-promote (`docs/adr/0013-icm-r0-
owner-rulings.md` decisions 6-7).

## Original intention

Work through the attention queue: gather context on the item at the front
of one of three fixed buckets, verify the underlying claim empirically,
recommend a category/state disposition and wait for maintainer direction,
escalate to an interview when the item is underspecified, and apply the
terminal disposition with its required artifact (agent brief, triage
notes, or a wontfix closure with or without an out-of-scope KB record).
Promoted into the N1 reference corpus as candidate **W30**
(`docs/gauntlet/contracts/N1.md`), with a full behavior-unit citation
trail archived at `docs/gauntlet/promoted-provenance/triage.md`
(`.sergeant/workflows/triage/index.md`). This ICM-R3 pass does not re-run
that N1 extraction; it applies the Placement and Bounded-Judgment ladders
on top of the already-cited content and checks the package's compliance
with ADR 0013's rulings, following the same method already proven at
ICM-R2 on `validate-and-ship` (`docs/gauntlet/runs/icm-r2/validate-and-
ship/adjudication-draft.md`).

## Current trigger and outcome

One linear five-stage sequence (`workflow.toml`: `10-gather-context`,
`20-verify`, `30-recommend`, `40-grill-if-underspecified`,
`50-apply-outcome`; the former `00-show-attention` stage was demoted and
folded into `10-gather-context` at N1 adjudication A4 — no additional
checkpoint argument existed beyond the §6.5 deterministic-machinery
boilerplate).

Trigger: an item is at the front of one of the three fixed attention
buckets (needs-triage oldest-first; needs-info items with fresh reporter
activity; qualifying external PRs), oldest first, or an explicit
natural-language maintainer request naming a specific item
(`docs/gauntlet/promoted-provenance/triage.md` `BU-P3-061`).

Outcome: the item reaches one of `needs-triage → {needs-info,
ready-for-agent, ready-for-human, wontfix}` with its outcome-specific
required artifact — a structured agent brief comment, templated triage
notes, a wontfix closing comment (with or without an out-of-scope KB
record depending on why), or (via `quick-override`) a maintainer-directed
state applied directly after confirmation.

## Driver and admission boundary

Driver: **stage actor**, all five stages. Admission boundary: **post-Work,
in-Work** — the workflow receives an already-scoped attention-queue item
(or an explicit maintainer request naming one) as its intent; it does not
itself decide whether triage-the-next-item should become durable Work, it
executes that already-defined intent to a terminal result. This passes the
execution-surface test (`docs/icm/convention.md` §2a: "would a human type
`sgt run '<intent>' --workflow triage`?") — yes, once an item is selected.

The one place this needs a closer look is `BU-P3-061`/`BU-P3-073`: the
workflow's trigger and its `quick-override` re-entry are both driven by
natural-language maintainer requests interpreted by the actor. This is
narrower than the PL-2 Captain discriminator in
`reference/proposal-icm-r-procedure-authority.md` §5.4 ("is the procedure's
job to decide what Work should exist?"): the interpretation here is scoped
entirely to selecting among the fixed triage state-machine actions on an
already-identified item (skip to a target state, or run the full
gather/verify/recommend/grill sequence), never to whether the current
activity itself should be direct or durable Work, nor to workflow
selection. This is the same shape already accepted for
`validate-and-ship/00-check-scope`'s "translate an ambiguous request into
a concrete pipeline flag" (`docs/gauntlet/runs/icm-r2/validate-and-ship/
adjudication-draft.md` BU-VAS-02, itself upheld against a REHOME
alternative in that record's own "Alternatives considered"). Not PL-2.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-TRI-01 | `docs/gauntlet/promoted-provenance/triage.md` workflow-level citations (`BU-P3-051/052/054-058/060/061`) — fixed state machine over issues and qualifying external PRs, ending in an agent-ready brief or another terminal disposition | PL-4 | package framing; no single J rung — narrowed per stage below | STAND | `triage` (workflow) |
| BU-TRI-02 | `10-gather-context/CONTEXT.md` — full read of item/prior notes/codebase exploration; already-implemented check; out-of-scope-KB concept-similarity match (`BU-P3-065`, `BU-P3-089`) | PL-5 | J2 (delegated: evidence inspection and the redundancy/prior-rejection verdict) | STAND | `10-gather-context` |
| BU-TRI-03 | `10-gather-context/CONTEXT.md` Helper section — three fixed attention buckets, oldest-first; needs-info-with-activity bucket; discovery filter excludes non-external PRs but not explicit requests (folded `00-show-attention`, N1 adjudication A4; `BU-P3-062/063/064`) | PL-6 | J5 (governing: bucket composition and ordering are fixed, not actor-chosen) | STAND — already correctly folded, no further placement change | `10-gather-context` (helper) |
| BU-TRI-04 | `20-verify/CONTEXT.md` — reproduce the bug or test the PR diff; report confirmed/failed/insufficient (`BU-P3-067`) | PL-5 | J2 (delegated: choice of reproduction/testing method) | STAND | `20-verify` |
| BU-TRI-05 | `30-recommend/CONTEXT.md` — propose category/state with reasoning and codebase summary, then wait for maintainer direction before any state-changing action (`BU-P3-066`) | PL-5 | J2 (delegated: the recommendation itself) + J5 (governing: no state-changing action proceeds before explicit maintainer direction — the gate that authorizes everything `50-apply-outcome` later does) | STAND | `30-recommend` |
| BU-TRI-06 | `40-grill-if-underspecified/CONTEXT.md` — underspecified items are escalated to a `grilling` interview, run live in this stage's own execution, not dispatched as a separate Work (`BU-P3-068`, adjusted: upstream pairs this with a `domain-modeling` procedure that does not exist in this repo, so sharpening folds into the same `grilling` session per `docs/icm/agents-invariant-dispositions.md` BU-1064) | PL-5 (stage) delegating to PL-3 (`grilling`, actor skill) | J2 (delegated: whether the item is underspecified after verification) | STAND | `40-grill-if-underspecified` |
| BU-TRI-07 | `docs/gauntlet/promoted-provenance/triage.md` workflow-level citations — `quick-override` (`BU-P3-073`) trusts an explicit maintainer directive to a specific state, skipping gather-context/recommend/grill after confirming intent; `resume` (`BU-P3-075`) never re-asks already-resolved questions on re-entry | PL-5 (re-entry variants of the same stage sequence, not separate stages — `CONTEXT.md` "Notes for reviewers") | J4 (explicit maintainer decision trusted directly, compatible with J5 — no governing constraint forbids skipping the sequence once the maintainer has explicitly named the target state) | STAND | package-level (re-entry variants) |
| BU-TRI-08 | `50-apply-outcome/CONTEXT.md` — outcome-specific required artifacts; out-of-scope KB written only for rejected enhancements (never bugs, never already-implemented closures); KB reconsideration removes/updates the record without reopening old closed issues (`BU-P3-069/070/071/072/074/090/091/092/093/096`) | PL-5 | J5 (governing: the KB write/no-write rules above are fixed, not actor-discretionary) + J4 (closing/posting/writing the KB record is authorized by the maintainer direction already obtained at `30-recommend` or `quick-override` — no separate consent gate is missing here, unlike the `validate-and-ship` push/pr/ci gap this pass specifically checked for) | STAND | `50-apply-outcome` |
| BU-TRI-09 | All five stage `CONTEXT.md` files carry only the generic "Judgment required" boilerplate paragraph; the workflow-level `CONTEXT.md` has no `## Authority envelope` section | N/A (authoring-format compliance, not a placement question) | J5 (governing requirement this package predates: ADR 0013 decision 4 + `docs/icm/convention.md` §6.1 — every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section, "always present... omission is never ambiguous"; every workflow `CONTEXT.md` carries `## Authority envelope`) | STAND (package identity correct; in-place content amendment required — see Surviving package design) | all five stage `CONTEXT.md` files + workflow `CONTEXT.md` |
| BU-TRI-10 | `CONTEXT.md:36` and `CONTEXT.md:40` — both cite a local `provenance.md` file for the A4 adjudication section and the full citation trail; no such file exists anywhere under `.sergeant/workflows/triage/` (it was archived to `docs/gauntlet/promoted-provenance/triage.md` at promotion; only `index.md` correctly points there) | N/A (dangling reference, not a placement question) | none — mechanical citation defect | FOLD (correct the reference in place; no placement change) | `CONTEXT.md` |
| BU-TRI-11 | `40-grill-if-underspecified/CONTEXT.md` Delegation section's citation of `grilling` (`skills/grilling/SKILL.md`, retirement rationale via R-NS-6) checked against `grilling`'s current, post-ICM-R2 content | N/A (citation-accuracy check, package-specific hint) | — | STAND, no correction needed (see "Grilling citation check" below) | — |

## Grilling citation check (BU-TRI-11, full record)

The task brief for this pass specifically flagged that `grilling` (already
reconciled at ICM-R2 as a STAND skill) had its own content change during
that pilot — a Failure-behavior citation fix and a new Bounded-judgment
section (`skills/grilling/SKILL.md` lines 43-91, `docs/icm/record-shapes.md`
worked-example cross-reference in `bounded-judgment.md`) — and asked
whether `triage`'s own citation of `grilling` still holds against that
current content, not the pre-pilot version.

`40-grill-if-underspecified/CONTEXT.md`'s Delegation section states: (1)
the stage's outcome is produced by running `grilling`
(`skills/grilling/SKILL.md`) to completion, live in this session, not by
dispatching a Work item; (2) `grilling` retired as a `.sergeant/workflows/`
package at the MVP-5 F2 execution-surface re-triage under North Star ruling
R-NS-6; (3) this also resolves the E3 dependency the stage previously
inherited from the retired package's WORKFLOW-IF-E3 classification.

Checked against `skills/grilling/SKILL.md` as it stands today:

- Claim (1) is still accurate. Nothing in the ICM-R2 Failure-behavior
  correction or the new Bounded-judgment section changes where or how
  `grilling` executes — it still runs live inside the invoking session,
  never via `sgt run`/Work dispatch (`skills/grilling/SKILL.md` "This
  skill must not do": "Run via `sgt run` or any durable Work dispatch").
  The new Failure-behavior text in fact directly supports `triage`'s use
  case: it explicitly anticipates being invoked from a headless/unattended
  context — exactly what a `triage` Work driving `40-grill-if-underspecified`
  as a dispatched stage execution is — and instructs degrading to a stated
  best-guess rather than presenting an unconfirmed guess as reached shared
  understanding, rather than forbidding the composition outright.
- Claim (2) is unaffected — it cites the retirement event itself
  (`docs/icm/re-homing-record-2026-08-12.md`), which the ICM-R2 pilot did
  not revisit or reverse; `grilling`'s SKILL.md frontmatter/provenance note
  still confirms the same port-from-workflow history.
- Claim (3) is unaffected for the same reason.
- `triage`'s own delegation text does not cite, quote, or depend on the
  specific passage that changed (the old `docs/environments/cerberus.md`
  citation the ICM-R2 pilot review corrected) — `triage` never referenced
  that citation itself, so the fix in `grilling` created no drift here.

Conclusion: **no correction needed.** `triage`'s citation of `grilling`
remains accurate against `grilling`'s current, post-ICM-R2 content.

## Surviving package design

No stage moves, merges, splits, or renames. All five stages, both re-entry
variants, and every already-cited N1 behavior unit remain correctly placed
at PL-4 (package) / PL-5 (each stage, including the delegation to `grilling`
at PL-3) / PL-6 (the folded `show-attention` helper). The package requires
**in-place content amendment**, not restructuring:

1. Add a `## Bounded judgment` section (per `docs/icm/convention.md` §7.3
   / `.sergeant/common/contexts/bounded-judgment.md`) to each of the five
   stage `CONTEXT.md` files, replacing the current `## Judgment required`
   boilerplate with named J2 delegations, J1 local choices, and J0
   escalation triggers specific to that stage — most of this is a direct
   restatement of judgment content the package's own Behavior contract
   sections already carry informally (see the J boundary column above).
   In particular, `30-recommend`'s "wait for maintainer direction before
   any state-changing action" belongs as an explicit J0 trigger (no
   direction yet given → `needs_input`), and `50-apply-outcome`'s KB
   write/no-write rules belong as explicit J5 clauses.
2. Add a `## Authority envelope` section to the workflow-level
   `CONTEXT.md` (per `convention.md` §7.2).
3. Correct the dangling `provenance.md` reference at `CONTEXT.md:36` and
   `CONTEXT.md:40`: the file was archived to
   `docs/gauntlet/promoted-provenance/triage.md` at promotion (`index.md`
   already points there correctly); the workflow-level `CONTEXT.md` should
   point to the same archived path instead of a filename that no longer
   exists in the live package tree. This same pattern (a live `CONTEXT.md`
   citing a local `provenance.md` that was archived elsewhere at
   promotion) recurs across other promoted N1 packages (e.g.
   `cross-repo-work/CONTEXT.md`) — it is recorded here as a
   `triage`-scoped fix; whether it should be corrected corpus-wide is a
   separate, broader remediation this producer does not scope-creep into
   here.

Neither amendment changes which package owns the behavior, so neither
triggers the REHOME/SPLIT/HARVEST draft-and-rehome step
(`docs/adr/0013-icm-r0-owner-rulings.md` decision 6). No `docs/gauntlet/
runs/icm-r3/triage/draft/` content is produced — this package's
classification did not conclude REHOME/SPLIT/HARVEST.

## Inputs and outputs

Inputs: as declared in each stage's own Inputs table — `10-gather-context`
reads only `../CONTEXT.md` (L1, first stage only, since `00-show-attention`
folded in); each subsequent stage reads its immediate predecessor's
`output/README.md` (L4). All five comply with `docs/icm/record-shapes.md`
§1a. No contract-bearing dependency was found undeclared.

Outputs: `output/README.md` in each stage declares its expected artifact
and disposition. Four of five (`10-gather-context` through
`40-grill-if-underspecified`) are `evidence` (Work-branch record only);
`50-apply-outcome`'s is `promote` (workflow deliverable), correctly
reflecting that it is the sole terminal stage. No violation found in the
Layer 4 declarations.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` — its structural and provenance identity does not
change. The three remediation items above are ordinary content edits to an
admitted workflow and should go through the repository's normal review
path for workflow content changes, not a new draft-and-promote cycle, per
`docs/icm/convention.md` §2 (the draft/admitted split governs *new or
substantially rewritten* content; adding a required section to an
already-admitted stage's `CONTEXT.md` is neither). This adjudication
record itself needs an independent reviewer pass
(`reference/proposal-icm-r-procedure-authority.md` §8.11) before its
findings are treated as settled.

## Alternatives considered

- **Classify the natural-language-trigger/`quick-override` behavior
  (`BU-P3-061`/`BU-P3-073`) as PL-2 Captain behavior**, on the theory that
  interpreting a maintainer's free-form request looks like intent shaping.
  Rejected: the interpretation is bounded entirely to selecting among a
  fixed set of already-defined triage-workflow actions on an
  already-identified item, never to whether triage-the-item should be
  direct or durable Work, nor to workflow selection — the PL-2
  discriminator in proposal §5.4 does not fire. Same reasoning already
  applied and upheld for `validate-and-ship/00-check-scope` at ICM-R2.
- **Treat `40-grill-if-underspecified`'s delegation to `grilling` as
  requiring correction**, given the task brief's explicit prompt to check
  it against grilling's post-ICM-R2 content. Rejected after verification
  (see "Grilling citation check" above): the claims `triage` makes about
  `grilling` are unaffected by what actually changed in that pilot pass.
- **Treat the missing Bounded-judgment/Authority-envelope sections as a
  SPLIT or HARVEST signal** (i.e., that the package's authority story is
  incoherent enough to need restructuring). Rejected: the missing sections
  are a uniform authoring-format gap already seen and remediated the same
  way at `validate-and-ship` (ICM-R2) — every decision they would name is
  already present, informally, in each stage's existing Behavior contract
  prose; this is in-place amendment, not a placement defect.
- **Silently correct the `provenance.md` reference corpus-wide** while
  producing this record. Rejected as out of scope: this pass's mandate is
  the `triage` package; the same defect recurring elsewhere is recorded as
  an observation for a separate remediation, not folded into this record's
  own disposition.

## Final disposition
STAND

## Validation evidence

- Source-valid: every existing behavior-unit citation in this package's
  five stage `CONTEXT.md` files and its workflow-level `CONTEXT.md` was
  read in full and traced to its already-archived N1 provenance
  (`docs/gauntlet/promoted-provenance/triage.md`); no new citation was
  fabricated for this pass.
- Placement-valid: every stage's already-recorded PL-5 rung (`actor-stage
  (§6.4, judgment)`) was independently re-derived from the Placement
  Ladder in this pass and confirmed; the `grilling` delegation was
  independently re-checked at PL-3 against `grilling`'s current content
  (see BU-TRI-11), not merely copied from the package's own prose.
- Authority-valid: **not yet** — this is precisely what BU-TRI-09 found
  missing. The package cannot be called authority-valid
  (`reference/proposal-icm-r-procedure-authority.md` §9.1 claim 3) until
  the Bounded-judgment/Authority-envelope remediation items land.
- Structurally valid: all five stage directories, their `output/README.md`
  declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly. The dangling
  `provenance.md` reference (BU-TRI-10) is a citation defect, not a
  structural-validity failure (it does not affect stage order, inputs, or
  outputs resolving).
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the package;
  `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims remain to be measured separately.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7); it
  does not self-promote.
