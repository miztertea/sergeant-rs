# Package adjudication: prototype

ICM-R3 full-reconciliation pass, `docs/adr/0013-icm-r0-owner-rulings.md`;
method per `reference/proposal-icm-r-procedure-authority.md` §8, scope per
§10.4; record shape per `docs/icm/record-shapes.md` §6. Producer pass
only — independent review is a separate step (proposal §8.11;
`docs/icm/convention.md` §6.2/6.3) and has not run yet. This record and
(if the verdict required it) any revised draft content are themselves
draft — neither is self-promoting (ADR 0013 decisions 6-7).

No prior ICM-R2 pilot hypothesis exists for this package (it was not in
the nine-package pilot corpus, `reference/proposal-icm-r-procedure-
authority.md` §10.3). This is a fresh investigation against current
content, following the ICM-R2-established method and depth
(`docs/gauntlet/runs/icm-r2/validate-and-ship/adjudication-draft.md`
used as the worked-example baseline for record shape and rigor).

## Original intention

Build a throwaway prototype to answer one specific design question —
either about logic/state or about UI appearance — then fold the
validated answer into production code while preserving the throwaway
work as a primary source on a non-`main` branch
(`.sergeant/workflows/prototype/CONTEXT.md` "Purpose"; `index.md`
description). Promoted into the N1 reference corpus as candidate **W21**
(`docs/gauntlet/contracts/N1.md`), decomposed from
`reference/sergeant-upstream/.agents/skills/prototype/{SKILL.md,LOGIC.md,
UI.md}` per `reference-corpus/synthesis.md` §1, with a full behavior-unit
citation trail archived at
`docs/gauntlet/promoted-provenance/prototype.md`. Re-triaged and
confirmed WORKFLOW (not skill) at `docs/icm/retriage-2026-08-11.md` line
26: "`00-select-branch` picks between logic/UI question types per the
actual design question; different questions produce structurally
different runs." This ICM-R3 pass does not re-run N1 extraction; it
applies the Placement and Bounded-Judgment ladders on top of the
already-cited N1 content, checks compliance with ADR 0013's rulings, and
independently re-verifies every citation against the current (still
unchanged) upstream source content.

## Current trigger and outcome

Six linear stages, with a documented conditional branch
(`workflow.toml`: `00-select-branch`, `10-record-question`,
`20L-build-logic`, `20U-build-variants`, `30-hand-off`, `40-capture`):

- **Trigger:** "The user wants to sanity-check whether a state model or
  logic feels right, or explore what a UI should look like"
  (`CONTEXT.md` "Trigger", repeated verbatim in every stage's own
  Purpose section).
- `00-select-branch` decides logic vs. UI (with a recorded heuristic
  fallback when the user is unreachable); `10-record-question` fixes the
  question before any code is written; `20L-build-logic` **or**
  `20U-build-variants` executes (mutually exclusive — both stage
  directories exist in the pinned `workflow.toml` stage list, but only
  one runs per Work, the other a documented no-op,
  `CONTEXT.md` "Notes for reviewers"); `30-hand-off` gives the user the
  artifact; `40-capture` folds the validated decision into production
  code and preserves the throwaway on a non-`main` branch.

Outcome: a validated design decision lands in production code, rewritten
to production standards, while the prototype itself survives as a
citable primary source outside `main`, and the answer/question pair is
recorded on the implementation issue or a commit (`40-capture`
behavior contract, `BU-P3-019`).

## Driver and admission boundary

Driver: **stage actor**, throughout. Admission boundary: **in-Work** —
every stage's Inputs table names only Layer-1 orientation (`00-select-
branch` only) or a prior stage's Layer-4 output; no stage negotiates
scope or work-existence with the user the way a Captain skill would.
Applying the execution-surface test (`docs/icm/convention.md` §2a):
"would a human type `sgt run '<the design question>' --workflow
prototype`?" — yes. The package receives an intent (a design question
that needs answering), runs multi-stage with judgment at every
checkpoint, and produces a result (production code + a preserved
throwaway branch) meaningful independent of the originating conversation
continuing. This confirms the package's own already-recorded rung
(every stage table entry in `CONTEXT.md` reads "actor-stage (§6.4,
judgment)") rather than merely repeating it.

**Rejected alternative reading:** `00-select-branch` and `10-record-
question` could look Captain-shaped ("ask the user," "record the
question") in isolation. They are not: both are fresh, durable,
stage-bound executions with declared Inputs/outputs and a bounded
outcome (which type; what question), not live dialogue about whether
Work should exist at all — the *decision to prototype* already happened
before this workflow is invoked (the trigger is stated as a precondition,
not a stage output). This matches the same reasoning the ICM-R2 pilot
already applied and rejected for `validate-and-ship`'s comparable
entry stages.

**Known consumer:** `worker-mission/20-implement/CONTEXT.md` line 29
names `prototype` as one of five disciplines `10-triage-and-route` may
select ("diagnose-bug, prototype, tdd, implement, or deepen-module") —
context composition today, not true nested-workflow invocation
(`docs/icm/convention.md` §4). This is a real, current, and accurate
delegation; verified by reading `worker-mission`'s own current content,
not assumed from the package-hint. No dangling or stale reference found
on the consumer side.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-PROTO-01 | `CONTEXT.md` / `index.md` — throwaway workflow that answers one design question, branching by question nature (`BU-P3-010`, `BU-P3-011`) | PL-4 | N/A (package-identity statement) | STAND | `prototype` (workflow) |
| BU-PROTO-02 | `00-select-branch/CONTEXT.md` — determine question type from prompt, code, or the user directly (`BU-P3-012`) | PL-5 | J2 (delegated: classify question type from available evidence) | STAND | `00-select-branch` |
| BU-PROTO-03 | `00-select-branch/CONTEXT.md` — UI questions route to the UI branch, whose target artifact shape (in-browser-switchable variants) is set as a consequence (`BU-P3-013`) | PL-5 | J1 (mechanical routing once the type is classified) | STAND | `00-select-branch` |
| BU-PROTO-04 | `00-select-branch/CONTEXT.md` — ambiguous + user unreachable falls back to a code-shape heuristic, with the assumption recorded explicitly rather than blocking (`BU-P3-014`) | PL-5 | J4 (ask the user first, when reachable) with a named **J2 fallback** (heuristic + recorded assumption) when J4 is unavailable — the package's own precedent for a J4→J2 ordered fallback, structurally identical in kind to `40-drive-gates`'s auto-fix/ask-user split the whole ladder generalizes from | STAND | `00-select-branch` |
| BU-PROTO-05 | `10-record-question/CONTEXT.md` — record the state model and exact question before any code is written (`BU-P3-021`) | PL-5 | J2 (delegated: capture the question precisely enough to check the eventual result against it) | STAND | `10-record-question` |
| BU-PROTO-06 | `20L-build-logic/CONTEXT.md` — build a small interactive terminal app driving the state model (`BU-P3-020`) | PL-5 | J2 | STAND | `20L-build-logic` |
| BU-PROTO-07 | `20L-build-logic/CONTEXT.md` — logic isolated behind a small pure interface, reusable independent of the throwaway shell (`BU-P3-022`) | PL-5 | J2 (design of the interface) bounded by J5 below | STAND | `20L-build-logic` |
| BU-PROTO-08 | `20L-build-logic/CONTEXT.md` — logic module purity (no I/O/console control flow); one-way dependency, TUI imports logic, never the reverse (`BU-P3-023`) | PL-5 | J5 (stage's own contract forbids the reverse dependency and I/O in the logic module — a hard constraint, not a style preference) | STAND | `20L-build-logic` |
| BU-PROTO-09 | `20L-build-logic/CONTEXT.md` — full-frame re-render on every update, never appended output (`BU-P3-024`, `BU-P3-025`, merged: same rule stated twice for the initial render and the post-action case) | PL-5 | J5 (stage contract states "must," not "should") | STAND | `20L-build-logic` |
| BU-PROTO-10 | `20L-build-logic/CONTEXT.md` — wired into the host project's existing task runner, runnable by name (`BU-P3-026`) | PL-5 | J2 (delegated: choose the task-runner entry/invocation name) | STAND | `20L-build-logic` |
| BU-PROTO-11 | `20U-build-variants/CONTEXT.md` — several structurally distinct UI variants on one route, switchable live (`BU-P3-029`) | PL-5 | J2 | STAND | `20U-build-variants` |
| BU-PROTO-12 | `20U-build-variants/CONTEXT.md` — prefer sub-shape A (mount in existing page) over sub-shape B (standalone route) (`BU-P3-030`) | PL-5 | J2 (delegated: choose sub-shape) with a named default preference | STAND | `20U-build-variants` |
| BU-PROTO-13 | `20U-build-variants/CONTEXT.md` — sub-shape B route follows the project's existing routing convention and is obviously named as a prototype (`BU-P3-031`) | PL-5 | J5 (governing: must not introduce a new routing convention; must be discoverable as throwaway) | STAND | `20U-build-variants` |
| BU-PROTO-14 | `20U-build-variants/CONTEXT.md` — default three variants, hard cap of five (`BU-P3-032`) | PL-5 | J5 (the cap itself, governing) + J2 (choosing a count within the bound) | STAND | `20U-build-variants` |
| BU-PROTO-15 | `20U-build-variants/CONTEXT.md` — variants must diverge structurally, not cosmetically; redo convergent drafts under an explicit divergence constraint (`BU-P3-033`) | PL-5 | J5 (governing: structural divergence is required) + J2 (judging whether divergence is real) | STAND | `20U-build-variants` |
| BU-PROTO-16 | `20U-build-variants/CONTEXT.md` — variant switcher gated off production builds (`BU-P3-034`) | PL-5 | J5 (governing, safety-relevant: prevents an accidental merge from exposing prototype UI to real users) | STAND | `20U-build-variants` |
| BU-PROTO-17 | `20U-build-variants/CONTEXT.md` — no real mutations; stub any write a variant needs (`BU-P3-038`) | PL-5 | J5 (governing: no real writes against production systems from prototype code) | STAND | `20U-build-variants` |
| BU-PROTO-18 | `30-hand-off/CONTEXT.md` — hand the user the URL and variant keys; expect cross-variant recombination as feedback, not a single pick (`BU-P3-035`) | PL-5 | J2 | STAND | `30-hand-off` |
| BU-PROTO-19 | `40-capture/CONTEXT.md` — fold the validated decision into real code; preserve the prototype as a primary source on a throwaway (non-`main`) branch; record answer + question on the issue or a commit (`BU-P3-019`) | PL-5 | J5 (governing: the prototype itself must never land on `main`) + **J4 gap** — see "The capture-trigger gap" below | STAND, with the J4 gap recorded as a required in-place amendment | `40-capture` |
| BU-PROTO-20 | `40-capture/CONTEXT.md` — logic branch capture: absorb the validated reducer/state-machine/function set; TUI shell preserved only on the throwaway branch (`BU-P3-027`) | PL-5 | J5 (governing, TUI must not merge) + same J4-gap asymmetry as BU-PROTO-19 (no explicit "the user confirmed" trigger, unlike the UI-branch capture units below) | STAND, in-place amendment recommended | `40-capture` |
| BU-PROTO-21 | `40-capture/CONTEXT.md` — TUI shell must never reach production (`BU-P3-028`) | PL-5 | J5 (governing) | STAND | `40-capture` |
| BU-PROTO-22 | `40-capture/CONTEXT.md` — sub-shape A capture: fold the winning variant into the existing page; remove other variants and the switcher from `main` (`BU-P3-036`) | PL-5 | **J4** (explicit: "the user has picked a winning variant" is the stated trigger) + J5 (non-winning variants/switcher must not survive on `main`) | STAND | `40-capture` |
| BU-PROTO-23 | `40-capture/CONTEXT.md` — sub-shape B capture: promote the winning variant to a real permanent route; remove throwaway route and switcher from `main` (`BU-P3-037`) | PL-5 | J4 + J5, same shape as BU-PROTO-22 | STAND | `40-capture` |
| BU-PROTO-24 | `40-capture/CONTEXT.md` — winning code rewritten to production standards, never merged as-authored (`BU-P3-039`) | PL-5 | J5 (governing quality gate) | STAND | `40-capture` |
| BU-PROTO-25 | All six stage `CONTEXT.md` files — uniform `## Judgment required` boilerplate; no stage names J2 decision classes, J1 local choices, or J0 escalation triggers in the `convention.md` §6.1 required shape | N/A (authoring-format compliance) | J5 (`docs/icm/convention.md` §6.1 + ADR 0013 decision 4: every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section, "always present... omission is never ambiguous" — a governing requirement this package predates) | STAND, in-place amendment required | all six stage `CONTEXT.md` files |
| BU-PROTO-26 | `CONTEXT.md` (L1) — no `## Authority envelope` section exists | N/A | J5 (`convention.md` §6.1: every workflow Layer-1 `CONTEXT.md` carries an `## Authority envelope` section) | STAND, in-place amendment required | `CONTEXT.md` |
| BU-PROTO-27 | `CONTEXT.md` (L1) "Provenance" section: `See provenance.md for the complete stage-to-behavior-unit mapping` — **no `provenance.md` exists in this package's own directory tree**; the actual citation trail lives at `docs/gauntlet/promoted-provenance/prototype.md` | N/A (dangling reference, not a placement question) | N/A | **FOLD** (correct the reference in place; no placement change) | `CONTEXT.md` |
| BU-PROTO-28 | Upstream source, never harvested: `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md` "Rules that apply to both," items 1-5 (throwaway naming/location near the real code; runnable with one command via whatever task runner the project already supports; no persistence by default, state in memory, scratch DB/file if a database is explicitly in question; skip polish — no tests, minimal error handling, no abstractions; surface the full relevant state after every action/switch) — **absent from `docs/gauntlet/promoted-provenance/prototype.md`'s citation list entirely**, and absent from current stage content except two narrow branch-specific instances (`BU-P3-024`/`025` cover "surface state" for the logic branch only; `BU-P3-026` covers "one command" for the logic branch only) — see "The harvested-but-unplaced cross-cutting rules" below | PL-5, cross-cutting across `20L-build-logic` and `20U-build-variants` (both branches build "prototype code") | J5 (governing, per the upstream source's own "must"/"skip"/"never" phrasing) once actually placed | **STAND at package level; content amendment required** — these five rules were never extracted as behavior units for this package at all, a harvest gap under proposal §8.4, not a placement dispute | `20L-build-logic`, `20U-build-variants` (both; `10-record-question` for the naming/location rule if code exists at that point) |

## The capture-trigger gap (BU-PROTO-19/20, full record)

`40-capture`'s own behavior contract for the **UI** branch explicitly
names the required J4 grant: `BU-P3-036`/`BU-P3-037` (BU-PROTO-22/23
above) state "the user has picked a winning variant" as the trigger.
For the **workflow-level** statement (`BU-P3-019`, BU-PROTO-19) and the
**logic-branch-specific** statement (`BU-P3-027`, BU-PROTO-20), the
trigger is only "once a prototype has answered its question" — no
stage names who or what decides the question has been answered, nor
does either statement's trigger say "the user confirmed."

**Rungs checked, for this producer's own classification of the gap's
current state:**

- **J5** — No governing constraint inside this package names who
  determines the question is answered for the logic branch or at the
  workflow level; the only explicit governing text on this point is the
  UI-branch-specific `BU-P3-036`/`037`.
- **J4** — No workflow-level or logic-branch stage names "the user
  confirms the answer" as a decision it consumes, unlike the UI branch.
- **J3** — No settled record elsewhither in this package states who
  validates "the question is answered" outside the UI sub-shapes.
- **J2** — `40-capture`'s own contract does not name "judging whether
  the question has been answered" as a delegated decision class.
- **J1** — Does not apply: whether the design question has actually
  been answered, before production code is rewritten and the throwaway
  is finalized, changes what lands in the codebase — not local,
  reversible, or non-contractual.

**Conclusion: this is a real gap, but a narrower one than
`validate-and-ship`'s BU-VAS-15 push/pr/ci gap** (ICM-R2 pilot,
`docs/gauntlet/runs/icm-r2/validate-and-ship/adjudication-draft.md`).
That gap reached full **J0** because the missing authority boundary sat
directly in front of an autonomous, externally-visible, hard-to-reverse
action (push/PR/CI) the dispatched Work itself could execute. Here,
`40-capture`'s own action stays on the Work branch — folding validated
code into the repository tree is not the same act as landing it on
`main`; per `reference/proposal-icm-r-procedure-authority.md` §9.7 and
this repository's normal review path, whatever `40-capture` produces
still passes through independent review before it is merged, treated as
settled, or published. The blast radius of guessing wrong here is
bounded by that downstream gate in a way `validate-and-ship`'s was not.
This does not make the gap disappear; it changes its disposition from
"stop and ask the owner before publishing this record" (J0) to "record
as a required in-place amendment, recommending the already-correct
UI-branch pattern be extended to the workflow-level and logic-branch
trigger language" — content work for whoever lands the remediation, not
a live owner decision this producer pass must escalate.

## The harvested-but-unplaced cross-cutting rules (BU-PROTO-28, full record)

`docs/icm/agents-invariant-dispositions.md` already adjudicated these
five upstream rules once, in a different lane: BU-1080 through BU-1084
(the AGENTS.md-invariant candidate pool) were each marked `not-adopted`
into `AGENTS.md`/repository doctrine with the disposition **"skill:
prototype... Prototype-shape rules belong to that workflow (published
WORKFLOW per retriage)."** That verdict — these rules belong inside the
`prototype` workflow, not repository-wide doctrine — is not
re-litigated here; it is confirmed correct by this pass's own
independent placement analysis (each rule is specific to prototype-code
construction, not broadly applicable across many unrelated tasks, so it
fails PL-1's "must apply broadly... independent of one trigger"
question and correctly lands at PL-5 instead). What this pass found is
that the correct destination was never actually written: **no stage's
`CONTEXT.md` contains these five rules**, and
`docs/gauntlet/promoted-provenance/prototype.md` never cites them at
all — they are not even present as rejected-but-considered candidates
in this package's own citation trail. Verified directly (`grep` across
`.sergeant/workflows/prototype/` found no match for "persistence,"
"skip the polish," "one command to run" outside the logic branch, or
"located close to where it will actually be used").

This is a harvest gap, not a placement dispute (proposal §8.4's
Harvest step precedes Normalize/Placement; a unit that was never
extracted cannot have been correctly or incorrectly placed). The
concrete remediation:

1. Item 5 ("surface the full relevant state") is already covered for
   the logic branch (`BU-P3-024`/`025`) but has no analog for
   `20U-build-variants` (surfacing state on every variant switch,
   distinct from the already-covered "no real mutations" rule,
   `BU-P3-038`).
2. Item 2 ("one command to run") is already covered for the logic
   branch (`BU-P3-026`) but has no analog for `20U-build-variants`
   (most web UI prototypes are already served by the project's existing
   dev-server command, so this may turn out to need no separate
   statement once actually drafted — a J1 authoring judgment for
   whoever lands the amendment, not a placement question).
3. Items 1, 3, and 4 (throwaway naming/location; no persistence by
   default; skip polish) currently have no analog in either branch and
   should be added, most naturally to both `20L-build-logic` and
   `20U-build-variants` (both are where "prototype code" actually gets
   written), with item 1's naming/location half potentially also
   relevant to `10-record-question` if code scaffolding begins there.

## Surviving package design

No stage moves, merges, splits, or renames. The six-stage sequence
(including the documented `20L`/`20U` mutually-exclusive branch), both
sub-shapes at the UI branch, and every already-cited N1 behavior unit
remain correctly placed at PL-4 (package) / PL-5 (each stage). The
package requires **in-place content amendment**, not restructuring:

1. Add a `## Bounded judgment` section (per `convention.md` §7.3 /
   `bounded-judgment.md`) to each of the six stage `CONTEXT.md` files,
   replacing/supplementing the current `## Judgment required`
   boilerplate with named J2 delegations, J1 local choices, and J0
   escalation triggers — most of this is a direct restatement of
   judgment content this package's Behavior contract sections already
   carry informally (see the J-boundary column above, derived from that
   existing prose) (BU-PROTO-25).
2. Add a `## Authority envelope` section to the workflow-level
   `CONTEXT.md` (per `convention.md` §7.2) (BU-PROTO-26).
3. Correct the dangling `provenance.md` reference in `CONTEXT.md`'s
   "Provenance" section to point at
   `docs/gauntlet/promoted-provenance/prototype.md`, the file that
   actually carries the citation trail (BU-PROTO-27).
4. Extend the explicit "the user confirms the answer" trigger language
   already present for the UI sub-shapes (`BU-P3-036`/`037`) to the
   workflow-level (`BU-P3-019`) and logic-branch (`BU-P3-027`) capture
   statements, so `40-capture`'s J4 grant is uniform across both
   branches rather than UI-only (BU-PROTO-19/20, "The capture-trigger
   gap" above).
5. Harvest and place the five never-extracted "Rules that apply to
   both" from the upstream `SKILL.md`, per the three concrete items
   under "The harvested-but-unplaced cross-cutting rules" above
   (BU-PROTO-28).

None of these five amendments changes which package owns the behavior,
so none triggers this ADR's REHOME/SPLIT/HARVEST draft-and-rehome step
(`docs/adr/0013-icm-r0-owner-rulings.md` decision 6; task brief). They
are recorded here as the concrete remediation this adjudication found,
for the owner/reviewer to schedule — this producer pass does not apply
them to the live package.

## Inputs and outputs

Inputs: as declared in each stage's own Inputs table — all six comply
with `record-shapes.md` §1a (verified during Inventory: `00-select-
branch` names only `../CONTEXT.md` as the first stage's L1 orientation
input; every later stage names exactly the immediately preceding
stage's output, including `30-hand-off`, which correctly names *both*
`20L-build-logic/output/README.md` and `20U-build-variants/output/
README.md` since only one of the two actually produced an artifact for
any given run). No contract-bearing dependency was found undeclared.
The dangling `provenance.md` prose reference (BU-PROTO-27) is not a
declared Inputs-table entry and is exactly the kind of unresolved
reference §1a rule 1 asks a reviewer to catch — the same shape as the
`route-review-findings` finding in the ICM-R2 `validate-and-ship` pass.

Outputs: `output/README.md` in each stage declares its expected
artifact and disposition. Five of six are `evidence` (Work-branch
record only); `40-capture`'s is `promote` (workflow deliverable),
correctly reflecting that it is the terminal stage. `docs/gauntlet/
promoted-provenance/prototype.md`'s own "Promotion note" already flags
that `40-capture` declares `promote` with no finalize step, one of "30
of 34 N1 packages in that shape" per `docs/icm/promotion-spec-2026-08-11.md`
§1 — this pass re-confirms that note is still accurate and leaves the
finalize-step question to human review at merge time, unchanged from
the prior curation act.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (not a draft) — its structural and provenance
identity does not change. The five remediation items above are ordinary
content edits to an admitted workflow and should go through this
repository's normal review path for workflow content changes, not a new
draft-and-promote cycle, per `docs/icm/convention.md` §2 (the
draft/admitted split governs *new or substantially rewritten* content;
adding required sections and correcting a dangling reference in an
already-admitted stage's `CONTEXT.md` is neither). Per ADR 0013
decision 6, only the promotable form of this change (once actually
made) needs independent review before it lands — this adjudication
record itself, being ICM-R3 evidence, needs this workstream's own
reviewer step (`reference/proposal-icm-r-procedure-authority.md` §8.11)
before its findings are treated as settled.

## Alternatives considered

- **REHOME `00-select-branch`/`10-record-question` to a Captain skill**,
  on the theory that "determine question type" and "ask the user" look
  conversational. Rejected: both stages already pass the execution-
  surface test (`convention.md` §2a) as fresh, durable, stage-bound
  executions with declared Inputs/outputs — the decision to prototype at
  all already happened before this workflow is invoked; this is the
  same reasoning the ICM-R2 pilot already applied and accepted for
  `validate-and-ship`'s comparable entry stages.
- **Treat the `20L`/`20U` mutually-exclusive branch as a PL-7 engine
  gap** requiring a conditional-stage schema extension. Rejected: the
  package's own `CONTEXT.md` "Notes for reviewers" already correctly
  classifies this as grammar pressure for a future extension, not a
  current gap, since the linear `workflow.toml` already represents it
  faithfully (both stage directories exist; the non-selected one is a
  documented no-op for that run) — this pass re-confirms that existing
  classification rather than re-litigating it, per the ladder's own
  first-honest-rung rule (proposal §4.8).
- **Escalate the capture-trigger gap (BU-PROTO-19/20) to full J0**,
  matching `validate-and-ship`'s BU-VAS-15 treatment. Rejected: unlike
  that case, no autonomous, externally-visible, hard-to-reverse action
  executes directly from this package's own stages — `40-capture`'s
  output still passes through independent review before merge or
  publication (§9.7) — so the gap is real but resolvable as an in-place
  content amendment (extend the already-correct UI-branch pattern),
  not an owner-level policy question this producer pass must escalate
  before publishing.
- **Fold the five "Rules that apply to both" (BU-PROTO-28) into
  `AGENTS.md`** as a PL-1 stable invariant instead of into the
  workflow's own stages. Rejected: `docs/icm/agents-invariant-
  dispositions.md` (BU-1080 through BU-1084) already ruled these belong
  to this workflow specifically, not repository-wide doctrine, and this
  pass's own independent PL-1 test (does the rule need to apply broadly
  across many unrelated tasks?) confirms that call — re-litigating an
  already-settled placement is out of scope; the actual finding here is
  that the correctly-identified destination was never written.
- **Silently add the missing content (bounded-judgment sections,
  authority envelope, corrected reference, capture-trigger language,
  the five harvested rules) on this producer's own authority**, rather
  than only recording it. Rejected per this Work's brief ("Produce the
  files and stop — you are the producer, not the reviewer"): a producer
  does not independently promote or apply its own output
  (`reference/proposal-icm-r-procedure-authority.md` §4.9); the
  remediation is recorded here for the owner/reviewer to schedule.

## Final disposition
STAND

## Validation evidence

- Source-valid: every existing behavior-unit citation in this package's
  six stage `CONTEXT.md` files and its L1 `CONTEXT.md`/`index.md` was
  read in full and traced to its already-archived N1 provenance
  (`docs/gauntlet/promoted-provenance/prototype.md`); the cited upstream
  source files (`reference/sergeant-upstream/.agents/skills/prototype/
  {SKILL.md,LOGIC.md,UI.md}`) were independently re-read in this pass
  and their current content still matches every citation verbatim — no
  drift found. One harvest gap was found (BU-PROTO-28: content present
  in the same primary source, never extracted); no citation was
  fabricated for this pass.
- Placement-valid: every stage's already-recorded PL-5 rung
  ("actor-stage (§6.4, judgment)") was independently re-derived from
  the Placement Ladder in this pass and confirmed, not merely copied
  from the package's own table; the package-level PL-4 rung was
  re-derived via the execution-surface test (`convention.md` §2a), also
  confirmed.
- Authority-valid: **not yet** — this is precisely what BU-PROTO-25/26
  (missing required sections) and BU-PROTO-19/20 (the capture-trigger
  asymmetry) found missing. The package cannot be called authority-valid
  (`reference/proposal-icm-r-procedure-authority.md` §9.1 claim 3) until
  the five remediation items under "Surviving package design" land.
- Structurally valid: all six stage directories, their `output/
  README.md` declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly. One
  structural defect found: the L1 `CONTEXT.md`'s own "Provenance"
  section names a file (`provenance.md`) that does not exist anywhere
  in this package's directory tree (BU-PROTO-27).
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the
  package; `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims (needs_input on a real/scripted J0 case,
  operation without Captain present) remain to be measured separately.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7); it
  does not self-promote.
