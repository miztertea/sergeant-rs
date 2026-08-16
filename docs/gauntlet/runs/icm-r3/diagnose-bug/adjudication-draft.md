# Package adjudication: diagnose-bug

ICM-R3 full-reconciliation pass, `docs/adr/0013-icm-r0-owner-rulings.md`;
method per `reference/proposal-icm-r-procedure-authority.md` §8 (Library
Reconciliation Method), §10.4 (ICM-R3 scope); record shape per
`docs/icm/record-shapes.md` §6. Producer pass only — independent review is
a separate step (§8.11 of the proposal; §6.2/6.3 of `docs/icm/
convention.md`) and has not run yet. This record is itself draft; it does
not self-promote (ADR 0013 decision 6, decision 7).

This package was not part of the ICM-R2 pilot corpus
(`docs/adr/0013-icm-r0-owner-rulings.md` decisions 8-9 name the nine pilot
packages; `diagnose-bug` is not one of them). This is its first pass under
either ladder. The proposal's own §12.7 records a hypothesis for this
class of package ("code-review, vet-external-skill, diagnose-bug,
resolving-merge-conflicts, and repo-to-icm appear naturally compatible
with already-defined intents, fresh stage execution, explicit artifacts,
and independent evidence... likely STAND after authority and handoff
rewrites, not guaranteed unchanged") — treated here as a hypothesis to
verify against the package's actual current content, not an answer to
restate.

## Original intention

Reproduce, isolate, prove, remediate, and verify a defect, ending only
once the repro is gone, a regression test passes (or its absent seam is
recorded as a finding), all temporary instrumentation is removed, and the
governing hypothesis is written down for the next debugger
(`.sergeant/workflows/diagnose-bug/CONTEXT.md` "Purpose"; `index.md`
description). Promoted into the N1 reference corpus as candidate **W20**
(`docs/gauntlet/contracts/N1.md`), decomposed from the upstream
`diagnosing-bugs` Claude Code skill
(`reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md`)
per `docs/icm/promotion-spec-2026-08-11.md`, with a full behavior-unit
citation trail archived at
`docs/gauntlet/promoted-provenance/diagnose-bug.md`. This pass does not
re-run that N1 extraction; it applies the Placement and Bounded-Judgment
ladders on top of the already-cited N1 content and checks the package's
compliance with ADR 0013's rulings.

## Current trigger and outcome

One entry point, one linear stage list (`workflow.toml`:
`10-build-feedback-loop`, `20-reproduce-and-minimize`, `30-hypothesize`,
`40-instrument`, `50-fix-with-regression-test`,
`60-cleanup-and-postmortem`).

Trigger: "Diagnose"/"debug this", or something reported broken, throwing,
failing, or slow (`CONTEXT.md` "Trigger"; `BU-P2-019`).

Outcome: a named, already-run, red-capable, deterministic, fast,
agent-runnable reproduction command is built; the bug is reproduced and
minimized to its load-bearing elements; 3-5 ranked falsifiable hypotheses
are generated and shown to the user; each surviving hypothesis is tested
with one targeted, tagged probe per prediction; a regression test is
written at a correct seam before the fix (or the seam's absence is
recorded as a finding) and the fix is proven against both the minimal and
original scenarios; and the run closes only once the repro is gone, the
test passes, instrumentation is removed, and the diagnosis is recorded —
with an optional, deliberately-timed handoff to an architecture-
improvement recommendation.

## Driver and admission boundary

Driver: **stage actor**, throughout. Admission boundary: **in-work** —
every stage's `CONTEXT.md` Inputs table names only Layer 1 orientation
(first stage) and the immediately preceding stage's Layer 4 output; none
depends on live conversational continuity, and every stage's own Behavior
contract describes what the actor does with an already-known defect, not
how to decide whether a defect investigation should exist. The package
passes the execution-surface test (`convention.md` §2a): "would a human
type `sgt run '<bug intent>' --workflow diagnose-bug`?" — yes, once a
concrete symptom is named; the workflow does not do "the same thing every
time" (its every stage is explicitly judgment-bearing, `## Judgment
required` on all six), so it is not a CLI verb, and no candidate `sgt`
surface already owns bisection/hypothesis-generation/instrumentation
judgment, so it is not PL-0/absorbed. Two bounded, non-blocking checkpoints
exist inside the run — Phase 1's "stop and ask for access/artifacts" when
no loop can be built (`BU-P2-028`), and Phase 3's "show the ranked
hypothesis list to the user... should not block progress if the user is
away" (`BU-P2-039`) — but neither makes live dialogue the package's
primary product; both are bounded questions raised *during* an
already-admitted execution, exactly the pattern PL-4's own text allows
("A workflow may ask a bounded question during execution, but conversation
cannot be its primary product"). This confirms the package's own
already-recorded placement (`CONTEXT.md`'s stage table already labels
every stage "actor-stage (§6.4, judgment)") rather than assuming it.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-DB-01 | `CONTEXT.md` (Trigger) — "Diagnose"/"debug this", or something reported broken, throwing, failing, slow; workflow-level trigger, `BU-P2-019` | PL-4 | J5 (contract-level: phases may be skipped only when explicitly justified, per `BU-P2-019`'s own text, not currently surfaced as a named J-clause — see BU-DB-09 below) | STAND | `diagnose-bug` (workflow) |
| BU-DB-02 | `10-build-feedback-loop/CONTEXT.md` — build a red-capable, deterministic, fast, agent-runnable reproduction command via a ranked construction ladder, tightened along three named axes, `BU-P2-021` through `BU-P2-029` | PL-5 | J2 (delegated: which construction strategy to attempt first and how to tighten it, `BU-P2-023`/`BU-P2-025`) with an explicit **J0** carve-out (`BU-P2-028`: if no loop can genuinely be built, stop, list what was tried, and ask the user for access/artifacts/instrumentation permission — never proceed to hypothesize without a loop, `BU-P2-030`) | STAND | `10-build-feedback-loop` |
| BU-DB-03 | `20-reproduce-and-minimize/CONTEXT.md` — confirm the loop reproduces the user's exact symptom, then shrink to the smallest still-red scenario, `BU-P2-031` through `BU-P2-035` | PL-5 | J2 (delegated: what to cut and in what order while minimizing, `BU-P2-033`) with a **J5** completion gate (must not proceed past this stage until both reproduction and minimization hold, `BU-P2-036` — a governing stage-contract prohibition, not a discretionary choice) | STAND | `20-reproduce-and-minimize` |
| BU-DB-04 | `30-hypothesize/CONTEXT.md` — generate 3-5 ranked, falsifiable hypotheses and show them to the user before testing, `BU-P2-037` through `BU-P2-039` | PL-5 | J2 (delegated: hypothesis generation and ranking, `BU-P2-037`/`BU-P2-038`) + J2 (delegated: proceeding without the user's re-ranking if the user is unavailable — an explicitly named non-blocking exception, `BU-P2-039`, distinct from a J0 escalation because the contract itself authorizes continuing) | STAND | `30-hypothesize` |
| BU-DB-05 | `40-instrument/CONTEXT.md` — one tagged probe per prediction, ordered tool preference, a measure-first branch for performance bugs, `BU-P2-040` through `BU-P2-043` | PL-5 | J2 (delegated: which instrumentation tool to reach for first and how to tag it, `BU-P2-041`/`BU-P2-042`) | STAND | `40-instrument` |
| BU-DB-06 | `50-fix-with-regression-test/CONTEXT.md` — write a regression test at a correct seam before the fix, or record the seam's absence as the finding, `BU-P2-044` through `BU-P2-047` | PL-5 | J2 (delegated: judging whether a candidate seam is load-bearing or too shallow, `BU-P2-045`) with a named fallback disposition when J2's answer is "no correct seam exists" (`BU-P2-046`: record the absence as a finding rather than silently skipping — a required outcome, not a discretionary one) | STAND | `50-fix-with-regression-test` |
| BU-DB-07 | `60-cleanup-and-postmortem/CONTEXT.md` — verify the closing checklist, then optionally hand off an architecture-improvement recommendation, `BU-P2-048`/`BU-P2-049` | PL-5 | J2 (delegated: judging whether the fix implicates an architectural change worth flagging, `BU-P2-049`) | STAND, with one in-place correction — see "The dangling handoff reference" below | `60-cleanup-and-postmortem` |
| BU-DB-08 | All six stage `CONTEXT.md` files — uniform `## Judgment required` boilerplate paragraph; no stage names its J2 decision classes, J1 local choices, or J0 escalation triggers in the required shape, though every Behavior contract already states the underlying judgment informally | N/A (authoring-format compliance, not a placement question) | J5 (`docs/adr/0013-icm-r0-owner-rulings.md` decision 4 + `docs/icm/convention.md` §6.1: every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section "always... omission is never ambiguous" — a governing requirement this package predates and does not yet satisfy) | STAND (package identity correct; in-place content amendment required — see "Surviving package design") | all six stage `CONTEXT.md` files |
| BU-DB-09 | `CONTEXT.md` (L1) — no `## Authority envelope` section exists | N/A | J5 (`convention.md` §6.1: every workflow Layer-1 `CONTEXT.md` carries an `## Authority envelope` section) | STAND, in-place amendment required | `CONTEXT.md` |
| BU-DB-10 | `60-cleanup-and-postmortem/CONTEXT.md` (`BU-P2-049`) — "hands off to the `/improve-codebase-architecture` skill with specifics" | N/A (dangling reference, not a placement question) | J2 as currently drafted implies a specific downstream surface that does not exist in this repository — see "The dangling handoff reference" below | FOLD (correct the reference in place; no placement change to this package) | `60-cleanup-and-postmortem/CONTEXT.md` |
| BU-DB-11 | `60-cleanup-and-postmortem/output/README.md` — declares a `promote` disposition but the stage names no deterministic finalize step (curation note already recorded at `docs/gauntlet/promoted-provenance/diagnose-bug.md` line 71, per `convention.md` §1a open question 1) | PL-6 (the missing piece, if built, would be a deterministic finalize helper) | N/A — pre-existing, already-recorded observation, not newly found here | Parked, not resolved by this pass (see "Alternatives considered") | `60-cleanup-and-postmortem` |

## The dangling handoff reference (BU-DB-10, full record)

`60-cleanup-and-postmortem/CONTEXT.md`'s Behavior contract (citing
`BU-P2-049`) instructs the actor to hand off to "the
`/improve-codebase-architecture` skill" when the postmortem finds an
architectural cause. That string was carried over verbatim from the
upstream Claude Code skill's own text
(`reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md`,
Phase 6) during N1 extraction. No skill or workflow by that name, or
functionally equivalent to it, exists anywhere in this repository:

```
$ grep -rn "improve-codebase-architecture" . (excluding .git and
  reference/sergeant-upstream)
.sergeant/workflows/diagnose-bug/60-cleanup-and-postmortem/CONTEXT.md
docs/gauntlet/promoted-provenance/diagnose-bug.md
docs/gauntlet/runs/n2-run4/... (a prior, non-current draft snapshot)
```

`skills/` currently contains only `estate-navigation`, `grill-with-docs`,
`grilling`, and `sergeant-help` (§12.8's already-rehomed baseline set).
None is an architecture-improvement skill. This is the same class of
finding as ICM-R2's `validate-and-ship` pilot review found for its
`route-review-findings` reference (`docs/gauntlet/runs/icm-r2/
validate-and-ship/adjudication-draft.md`, BU-VAS-10): a `@@`-shaped or
slash-command-shaped forward reference to a procedure that was never
built in this repository, surviving from upstream or historical text.

**Rungs checked:** J5 — no policy requires this specific handoff string;
J4 — no user or Work decision names it; J3 — no settled record in this
repository defines an `/improve-codebase-architecture` surface; J2 — this
stage's own contract is what asserts the reference, so it cannot be the
settling authority for whether the referenced thing exists. **Conclusion:**
this is not a J0 (no material, risk-changing decision is blocked on it —
the stage's actual required behavior, "flag the architectural
recommendation for the next debugger," is fully satisfiable without a
named handoff target) and not an engine-gap (nothing about it requires a
new runtime fact; `convention.md` §4 rule 1 already rules that a forward
reference to a procedure that does not currently exist is a scope
violation, not evidence for inventing one). It is a **FOLD**: correct the
stage text in place to describe the actual current behavior — record the
architectural finding and recommendation directly in the stage's Layer 4
output (already declared `promote`, so it survives into the merge) rather
than naming a specific downstream skill invocation that does not exist —
with a forward note if such a skill is ever built later.

## Surviving package design

No stage moves, merges, splits, or renames. The six-stage linear
sequence, its single entry point, and every already-cited N1 behavior
unit remain correctly placed at PL-4 (package) / PL-5 (each stage). The
package requires **in-place content amendment**, not restructuring:

1. Add a `## Bounded judgment` section (per `convention.md` §7.3 /
   `.sergeant/common/contexts/bounded-judgment.md`) to each of the six
   stage `CONTEXT.md` files, replacing the current `## Judgment required`
   boilerplate with named J2 delegations, the one J5 completion gate
   (`20-reproduce-and-minimize`), and the one named J0 escalation
   (`10-build-feedback-loop`'s "no loop can be built" case) — this is
   almost entirely a direct restatement of judgment content each stage's
   Behavior contract already carries informally (see the J boundary
   column above, derived from that existing prose).
2. Add a `## Authority envelope` section to the workflow-level
   `CONTEXT.md` (per `convention.md` §7.2), naming: the workflow may
   decide construction strategy, minimization cuts, hypothesis ranking,
   instrumentation tooling, seam adequacy, and architecture-handoff
   judgment (all J2, per the table above); the workflow may not decide to
   proceed past Phase 1 without a red-capable loop or past Phase 2 without
   both reproduction and minimization (J5 gates); and the one human/
   Captain-visible checkpoint is Phase 3's ranked-hypothesis display,
   which is advisory and non-blocking by the package's own text
   (`BU-P2-039`).
3. Correct the dangling `/improve-codebase-architecture` reference in
   `60-cleanup-and-postmortem/CONTEXT.md` per BU-DB-10 above: state the
   actual current behavior (record the architectural finding in the
   stage's own `promote`-disposition output) rather than naming a skill
   that does not exist in this repository.
4. Leave BU-DB-11 (the missing deterministic finalize step for
   `60-cleanup-and-postmortem`'s `promote` output) as a parked,
   already-recorded observation — see "Alternatives considered" for why
   this pass does not resolve it.

None of these four amendments changes which package owns the behavior, so
none triggers ADR 0013's REHOME/SPLIT/HARVEST draft-and-rehome step
(decision 6; task brief: "If your classification concludes the package
should be rewritten (REHOME/SPLIT/HARVEST), also write the revised draft
content"). They are recorded here as the concrete remediation this
adjudication found, for the owner/reviewer to schedule — the same
disposition ICM-R2's `validate-and-ship` pilot pass reached for its own
in-place-amendment findings.

## Inputs and outputs

Inputs: as declared in each stage's own Inputs table — verified during
Inventory. `10-build-feedback-loop` inputs only `../CONTEXT.md` (L1,
first stage); each subsequent stage inputs exactly the immediately
preceding stage's `output/README.md` (L4). No contract-bearing dependency
was found undeclared, and no stage inputs a later stage's output or its
own `CONTEXT.md` in violation of `record-shapes.md` §1a rules 2-3.

Outputs: `output/README.md` in each stage declares its expected artifact
and disposition. Five of six are `evidence` (Work-branch record only);
`60-cleanup-and-postmortem`'s is `promote` (workflow deliverable),
correctly reflecting that it is the workflow's true closing stage per
`workflow.toml`'s own stage order — matching the already-recorded
curation note at `docs/gauntlet/promoted-provenance/diagnose-bug.md` line
71. No other violation found in the Layer 4 declarations.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (not a draft) — its structural and provenance
identity does not change under this disposition. The four remediation
items above are ordinary content edits to an admitted workflow and should
go through this repository's normal review path for workflow content
changes, not a new draft-and-promote cycle, per `docs/icm/convention.md`
§2 (the draft/admitted split governs *new or substantially rewritten*
content; adding a required section to an already-admitted stage's
`CONTEXT.md`, or correcting one dangling reference, is neither). Per ADR
0013 decision 6, only the promotable form of this change (once actually
made) needs independent review before it lands — this adjudication record
itself, being ICM-R3 evidence, needs its own independent-reviewer step
(`reference/proposal-icm-r-procedure-authority.md` §8.11) before its
findings are treated as settled.

## Alternatives considered

- **Treat the missing `## Bounded judgment` sections as sufficient reason
  to REHOME or SPLIT the package**, on the theory that a package failing
  an ADR 0013 authoring-format requirement is not yet "correct" and
  should be redrafted wholesale. Rejected: the format gap is a required
  in-place amendment (§7.3 of the proposal), not evidence the current
  surface is the wrong one — every stage's underlying judgment content is
  already sound and already PL-5-correct; restating it in the canonical
  section shape does not change which package or stage owns it, exactly
  as `validate-and-ship`'s ICM-R2 pass concluded for the same class of
  gap.
- **Treat Phase 3's "show the ranked hypothesis list to the user" as a
  Captain-shaped conversational checkpoint, and split the package to move
  live user interaction to a skill.** Rejected: the checkpoint is
  explicitly non-blocking by the package's own already-cited text
  (`BU-P2-039`: "should not block progress if the user is away") — it is
  a bounded question raised during an already-admitted execution, exactly
  the pattern PL-4 names as legal ("a workflow may ask a bounded question
  during execution, but conversation cannot be its primary product"), not
  evidence the workflow's primary product is dialogue.
- **Treat the dangling `/improve-codebase-architecture` reference as an
  engine-gap (PL-7) claim** — i.e., argue the runtime needs a new
  cross-workflow handoff capability. Rejected: nothing about the gap
  requires the runtime to own a new durable fact; the actual required
  behavior (record the architectural finding for the next debugger) is
  already satisfiable through the stage's own `promote`-disposition
  output without any new mechanism. Lower rungs (correcting the stage
  text) have not been attempted yet, so PL-7 is unreached per the
  ladder's own first-honest-rung rule (proposal §4.8).
- **Resolve BU-DB-11's missing finalize step for this package specifically**,
  since it was already flagged in the N1 provenance record. Rejected as
  this pass's own scope: `convention.md` §1a's own open-questions section
  already records that the finalize step is "a canonical execute-stage
  workload once `kind = "execute"` exists" and that this is one of "the
  corpus's 30 packages in that shape" — a cross-cutting convention gap
  shared by many packages, not something specific to `diagnose-bug`'s own
  placement or authority. Resolving it package-by-package here would be
  exactly the file-shape-mirroring failure `record-shapes.md` §6 rule 4
  warns against; it is left as a parked, already-recorded observation for
  whichever pass addresses the finalize-step convention gap corpus-wide.
- **Rewrite `BU-P2-019`'s "phases may be skipped only when explicitly
  justified" as a new, separate J-clause distinct from the per-stage
  gates already covered by BU-DB-02/03.** Rejected: this is the same
  governing constraint already expressed at stage granularity (Phase 1's
  J0 "no loop, no Phase 2" and Phase 2's J5 "must not proceed... until
  both hold") — restating it a third time at the workflow level would be
  the "identical generic reasons copied across lower rungs" pattern
  §5.9 of the proposal names as rejection evidence, not genuine
  additional judgment. The workflow-level `## Authority envelope`
  (remediation item 2) names it once, at the coarsest grain, and the
  stage-level sections do not repeat it verbatim.

## Final disposition
STAND

## Validation evidence

- Source-valid: every existing behavior-unit citation in this package's
  six stage `CONTEXT.md` files and its `CONTEXT.md`/`index.md` was read in
  full and traced to its already-archived N1 provenance
  (`docs/gauntlet/promoted-provenance/diagnose-bug.md`); no new citation
  was fabricated for this pass; the one new textual observation (the
  dangling `/improve-codebase-architecture` reference) was independently
  confirmed by a repository-wide grep, not assumed from the provenance
  file's silence on it.
- Placement-valid: every stage's already-recorded PL-5 rung (`actor-stage
  (§6.4, judgment)`) was independently re-derived from the Placement
  Ladder in this pass and confirmed, not merely copied from the package's
  own table; the workflow-level PL-4 rung was independently checked
  against the execution-surface test (`convention.md` §2a) rather than
  assumed from the proposal's own §12.7 hypothesis.
- Authority-valid: **not yet** — this is precisely what BU-DB-08/09/10
  found missing. The package cannot be called authority-valid
  (`reference/proposal-icm-r-procedure-authority.md` §9.1 claim 3) until
  the four remediation items under "Surviving package design" land.
- Structurally valid: all six stage directories, their `output/README.md`
  declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly. No `@@name`
  references exist anywhere in this package (grep-confirmed), so there is
  no delegated content whose current, possibly-already-amended state
  needed separate verification for this wave.
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the package;
  `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims (needs_input on a real/scripted J0 case,
  operation without Captain present) remain to be measured separately.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7); it
  does not self-promote.
