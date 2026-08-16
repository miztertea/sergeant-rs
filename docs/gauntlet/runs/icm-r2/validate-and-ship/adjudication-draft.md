# Package adjudication: validate-and-ship

ICM-R2 pilot package, `docs/adr/0013-icm-r0-owner-rulings.md` decisions 8-9;
method per `reference/proposal-icm-r-procedure-authority.md` §8; record
shape per `docs/icm/record-shapes.md` §6. Producer pass only — independent
review is a separate step (§8.11 of the proposal; §6.2/6.3 of
`docs/icm/convention.md`) and has not run yet. This record and (if the
verdict required it) any revised draft content are themselves draft —
neither is self-promoting (ADR 0013 decision 6, decision 7).

## Original intention

The single final shipping boundary: validate a committed change through
the `no-mistakes` pipeline to a terminal outcome, routing every finding to
tracked work, without the validating actor ever editing the code under
review (`.sergeant/workflows/validate-and-ship/CONTEXT.md` "Purpose";
`index.md` description). Promoted into the current N1 reference corpus as
candidate **W18** (`docs/gauntlet/contracts/N1.md`,
`docs/icm/promotion-spec-2026-08-11.md`), with a full behavior-unit
citation trail already archived at
`docs/gauntlet/promoted-provenance/validate-and-ship.md`. This ICM-R2 pass
does not re-run that N1 extraction; it applies the newer Placement and
Bounded-Judgment ladders on top of the already-cited N1 content and checks
the package's compliance with ADR 0013's twelve rulings.

The proposal itself names this package's `40-drive-gates` stage as the
concrete precedent the whole Bounded-Judgment Ladder generalizes from
(§3.4, Finding ICMR-F4: "validate-and-ship/40-drive-gates already
distinguishes three kinds of gate finding... That is exactly the concept
needed").

## Current trigger and outcome

Two entry variants sharing one linear stage list (`workflow.toml`:
`00-check-scope`, `10-do-the-work`, `20-select-intent-transport`,
`30-start-run`, `40-drive-gates`, `50-reconcile-custody`, `60-close-out`):

- **Coordinator-launched entry**, at `20-select-intent-transport`: "Implementation,
  native tests, lint and independent review are complete and the
  coordinator has reached the approved shipping boundary."
- **Directly-invoked entry**, at `00-check-scope`: the user invokes
  `/no-mistakes` in the current session, with or without a task
  description.

Outcome: every pipeline gate resolved by exactly one response, every
actionable finding routed to a deduplicated owning-repo task, and a
terminal state (`checks-passed`, `passed`, `failed`, or `cancelled`)
reached without the validating actor editing the pipeline-owned worktree.

## Driver and admission boundary

Driver: **stage actor**, both entries. Admission boundary: **post-Work,
in-Work** — the coordinator-launched entry receives an already-admitted,
already-reviewed Work intent; the directly-invoked entry runs inside the
current interactive session but is itself durable multi-stage execution
(fresh execution per stage, `docs/icm/convention.md` §1a), not live
Captain dialogue about what Work should exist. Both entries pass the
execution-surface test (`convention.md` §2a): "would a human type `sgt run
'<intent>' --workflow validate-and-ship`?" — yes, for either entry, once
scope is established. This matches the workflow's own already-recorded
placement (its `CONTEXT.md` stage table already labels every stage
"actor-stage (§6.4, judgment)").

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-VAS-01 | `CONTEXT.md` (Purpose) — single final shipping boundary; actor never edits code | PL-4 | J5 (contract-level prohibition: actor never edits the pipeline-owned code) | STAND | `validate-and-ship` (workflow) |
| BU-VAS-02 | `00-check-scope/CONTEXT.md` — determine invocation mode; translate an ambiguous request into a concrete pipeline flag | PL-5 | J2 (delegated: mode determination and flag translation, `BU-P2-058/059`) | STAND | `00-check-scope` |
| BU-VAS-03 | `10-do-the-work/CONTEXT.md` — isolate and commit only task-scoped changes on a feature branch | PL-5 | J2 (delegated: which working-tree changes belong to the task, `BU-P2-060/061`) | STAND | `10-do-the-work` |
| BU-VAS-04 | `20-select-intent-transport/CONTEXT.md` — probe installed capability, decide the intent transport once, require explicit consent for the argv-exposing option | PL-5 (stage); PL-6 (the capability probe itself) | J5 (governing: intent content must never transit argv, `BU-P6-134`, `BU-P8-085`) + J4 (consent is the operator's explicit, per-run decision, not the actor's) | STAND | `20-select-intent-transport` |
| BU-VAS-05 | `20-select-intent-transport/CONTEXT.md` Helper section — readiness marker / launch reservation / isolated snapshot preconditions (folded `00-verify-readiness`, `10-acquire-launch-reservation`, `20-reserve-isolated-snapshot`, N1 adjudication A4) | PL-6 | J5 (governing: launch refuses on stale head, mismatched intent revision, or any non-`passed` review axis) | STAND | `20-select-intent-transport` (helper) |
| BU-VAS-06 | `20-select-intent-transport/CONTEXT.md` Helper section — repo-level pre-push drain-suite gate, re-homed from the demoted `repo-release-verification` package (N1 adjudication A6) | PL-6 (this repository's own git pre-push hook, mechanically gating every push in the repo, not workflow-scoped machinery) | J5 (governing: push blocked on failure unless `--no-verify`; fails closed if tooling is unavailable) | STAND — re-homing already correctly executed; no further placement change needed | `20-select-intent-transport` (helper) |
| BU-VAS-07 | `30-start-run/CONTEXT.md` — discover/reattach an in-flight run vs. start new; compose a sufficiently rich `--intent` | PL-5 | J2 (delegated: in-flight-run handling per `BU-P2-068`-`071`; intent composition per `BU-P2-073`) | STAND | `30-start-run` |
| BU-VAS-08 | `40-drive-gates/CONTEXT.md` — auto-fix/no-op findings driven on actor judgment; ask-user findings relayed verbatim, never resolved autonomously | PL-5 | J2 (auto-fix/no-op, `BU-P2-079/080`) with an explicit **J0** carve-out per finding (ask-user, `BU-P2-098/099`) — the canonical worked precedent this whole ladder generalizes from (proposal §3.4) | STAND | `40-drive-gates` |
| BU-VAS-09 | `40-drive-gates/CONTEXT.md` Helper section — deterministic finding-to-`td` routing (folded `60-route-findings`, N1 adjudication A4) | PL-6 | J5 (governing: severity/disposition deterministically fixes `td` routing and blocking eligibility, `BU-P6-023`) | STAND | `40-drive-gates` (helper) |
| BU-VAS-10 | `CONTEXT.md` "Relationships to other workflows" + `40-drive-gates/CONTEXT.md` — routing "is also produced in part by running **route-review-findings** to its own completion" | none resolvable as written — **dangling reference** | — | **FOLD** (correct the reference in place; no placement change to this package) | `40-drive-gates/CONTEXT.md`, `CONTEXT.md` |
| BU-VAS-11 | `50-reconcile-custody/CONTEXT.md` — process structured `branch_sync` state (sync/continue/recover-custody), never improvised git surgery | PL-5 | J5 (governing: never reset/stash/force/replace branch) + J2 (choosing among the three structured remediation paths) | STAND | `50-reconcile-custody` |
| BU-VAS-12 | `60-close-out/CONTEXT.md` — stop at `checks-passed`; fix-and-redrive on `failed`/`cancelled`; durable handover log | PL-5 | J2 (delegated: diagnosing and fixing what blocked the gate before redriving) + J5 (governing: never poll/wait for merge) | STAND | `60-close-out` |
| BU-VAS-13 | All seven stage `CONTEXT.md` files — uniform `## Judgment required` boilerplate paragraph, no stage names J2 decision classes, J1 local choices, or J0 escalation triggers in the required shape | N/A (authoring-format compliance, not a placement question) | J5 (ADR 0013 decision 4 + `docs/icm/convention.md` §6.1: every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section "always... omission is never ambiguous" — governing requirement this package predates and does not yet satisfy) | STAND (package identity correct; in-place content amendment required — see Alternatives considered) | all seven stage `CONTEXT.md` files |
| BU-VAS-14 | `CONTEXT.md` (L1) — no `## Authority envelope` section exists | N/A | J5 (`convention.md` §6.1: every workflow Layer-1 `CONTEXT.md` carries an `## Authority envelope` section) | STAND, in-place amendment required | `CONTEXT.md` |
| BU-VAS-15 | `scripts/gate.sh:202` (`--skip push,pr,ci`) vs. `30-start-run/CONTEXT.md` and `40-drive-gates/CONTEXT.md` (silent on push/PR/CI entirely) | N/A (this is a missing-authority-boundary gap inside an already-PL-5 stage pair, not a placement question) | **J0 — not delegated, conflicting, or risk-changing** (see "The push/pr/ci gap" below for the full checked-rung record) | STAND at package-identity level; the missing J-clause is **not drafted by this producer** — see below | `20-select-intent-transport/CONTEXT.md` and/or `40-drive-gates/CONTEXT.md` (owner TBD) |

## The push/pr/ci gap (BU-VAS-15, full record)

This package's own content contains **no explicit J classification anywhere**
for whether a dispatched `validate-and-ship` Work may autonomously push a
branch, open a pull request, or trigger CI. The only place that authority
question is currently answered is `scripts/gate.sh` — a repository-local
wrapper script, outside this workflow's own `.sergeant/workflows/
validate-and-ship/` tree, invoked manually per `docs/DEVELOPMENT.md`:

```
scripts/gate.sh:202:  exec no-mistakes axi run --intent "$intent" --skip push,pr,ci "$@"
```

`docs/DEVELOPMENT.md:105` confirms the flag is load-bearing policy, not an
incidental default: `` `scripts/gate.sh "<intent>"` runs the no-mistakes
pipeline (`--skip push,pr,ci`; push/PR handled manually). `` Nothing in
`30-start-run/CONTEXT.md` (which composes the `axi run --intent "..."`
invocation, `BU-P2-062`/`BU-P2-072`-`074`) or `40-drive-gates/CONTEXT.md`
(which drives every gate to a terminal outcome) names `push`, `pr`, `ci`,
`--skip`, or any authorization requirement for autonomous publication.
A Work dispatched directly against this workflow's own stages — the path
`scripts/gate.sh` exists precisely to wrap — never executes that flag and
has no stage-level instruction telling it the flag exists, why it matters,
or who may waive it.

This is confirmed live, not hypothetical, by two independent runs already
in this repository's own gauntlet history:

- `docs/gauntlet/runs/path-to-mac-2026-08-15/retrospective.md` §3.1: a
  dispatched gate Work (W6), whose brief told it to run
  `validate-and-ship` directly rather than through `scripts/gate.sh`,
  pushed `sergeant/<work-id>` into the primary checkout; PR and CI were
  skipped only because that estate's `origin` happened to be a local
  path, not because anything in the workflow stopped them. The
  retrospective's own §7 item 1 names this "a product gap" left
  unresolved: "`scripts/gate.sh` is what carries `--skip push,pr,ci`. A
  `validate-and-ship` Work driving the pipeline directly gets none of
  them."
- `docs/gauntlet/runs/macbook-arrival-2026-08-15/retrospective.md` §3:
  "**#123 materialized, not just predicted.** The dispatched WD gate Work
  autonomously pushed its own branch, opened PR #141 directly against
  `main`, and ran CI to completion" — on an estate whose `origin` was a
  real GitHub host, with nothing in the workflow content stopping it.

**Rungs checked (bounded-judgment.md order), for this producer's own
classification of the gap's current state — not a resolution of the
underlying policy:**

- **J5** — No governing constraint inside this workflow's own content
  requires or forbids autonomous push/PR/CI. The only governing text that
  exists (`scripts/gate.sh`'s hard-coded `--skip`) lives outside the
  package and is never invoked by the dispatched-Work path.
- **J4** — No explicit user or bound-Work decision is visible to
  `30-start-run` or `40-drive-gates` that would authorize or forbid
  autonomous publication; a Work's brief could in principle grant this,
  but no stage names it as a decision class it consumes.
- **J3** — No settled record inside this package states who may
  authorize push/PR/CI when driven as a dispatched Work.
- **J2** — No stage's Behavior contract names "whether to push / open a
  PR / trigger CI" as a delegated decision class.
- **J1** — Does not apply: whether a Work autonomously publishes a
  branch, opens a PR, and triggers CI to completion is exactly the kind
  of choice `bounded-judgment.md`'s own J1 definition excludes — it
  changes public/external-facing behavior and is not local or reversible
  once CI and reviewers have been notified.

**Conclusion: J0**, honestly, as the package stands today — not a design
recommendation, a description of the gap. Per this Work's own brief, the
underlying policy question (should `validate-and-ship`, when driven as a
dispatched Work rather than through `scripts/gate.sh`, ever autonomously
push/open a PR/trigger CI, and under what recorded consent if so) is a
live, separate owner decision this producer does not make. What this
record does do is what `bounded-judgment.md`'s J0 procedure requires of an
actor at J0: record the unresolved decision, state which rungs were
checked and why none settled it, and preserve the evidence gathered — all
done above — without offering a recommended answer this producer is not
entitled to author on the owner's behalf.

## Surviving package design

No stage moves, merges, splits, or renames. The seven-stage linear
sequence, both entry variants, and every already-cited N1 behavior unit
remain correctly placed at PL-4 (package) / PL-5 (each stage) / PL-6 (each
identified helper). The package requires **in-place content amendment**,
not restructuring:

1. Add a `## Bounded judgment` section (per `convention.md` §7.3 /
   `bounded-judgment.md`) to each of the seven stage `CONTEXT.md` files,
   replacing (or supplementing, per house style once ICM-R1's template
   lands) the current `## Judgment required` boilerplate with named J2
   delegations, J1 local choices, and J0 escalation triggers specific to
   that stage — most of this is a direct restatement of judgment content
   this package's Behavior contract sections already carry informally
   (see the J boundary column above, derived from that existing prose).
2. Add a `## Authority envelope` section to the workflow-level
   `CONTEXT.md` (per `convention.md` §7.2).
3. Correct the dangling `route-review-findings` reference in
   `CONTEXT.md` and `40-drive-gates/CONTEXT.md`: the package this
   package's own `40-drive-gates` currently claims to delegate part of
   its outcome to was retriaged to CLI-SURFACE / NET-NEW-SURFACE and
   never built (`docs/icm/retriage-2026-08-11.md` line 52,
   `docs/icm/re-homing-record-2026-08-12.md` line 29: unbuilt `sgt review
   route-findings`/`sgt gate clear` verb candidates). No package or draft
   named `route-review-findings` exists under `.sergeant/workflows/` or
   `.sergeant/drafts/workflows/`. The actual current mechanism is the
   `sgt-no-mistakes-finding` deterministic routing already folded into
   `40-drive-gates` as its own helper (BU-VAS-09 above) — the reference
   should describe that, not a sibling workflow that does not exist.
4. Leave a citable placeholder at the push/pr/ci gap (BU-VAS-15) for the
   owner's eventual ruling; do not invent the J-clause's content.

None of these four amendments changes which package owns the behavior, so
none triggers this ADR's REHOME/SPLIT/HARVEST draft-and-rehome step
(`docs/adr/0013-icm-r0-owner-rulings.md` decision 6; task brief). They are
recorded here as the concrete remediation this adjudication found, for the
owner/reviewer to schedule.

## Inputs and outputs

Inputs: as declared in each stage's own Inputs table (all seven already
comply with `record-shapes.md` §1a — verified during Inventory). No
contract-bearing dependency was found undeclared. The `@@`-style reference
to `route-review-findings` is prose delegation, not a declared Inputs-table
entry, and is exactly the kind of unresolved reference §1a rule 1 asks a
reviewer to catch.

Outputs: `output/README.md` in each stage declares its expected artifact
and disposition. Six of seven are `evidence` (Work-branch record only);
`60-close-out`'s is `promote` (workflow deliverable), correctly reflecting
that it is the terminal stage since `90-handover-log` was folded into it.
No violation found in the Layer 4 declarations.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (not a draft) — its structural and provenance
identity does not change. The four remediation items above are ordinary
content edits to an admitted workflow and should go through this
repository's normal review path for workflow content changes, not a new
draft-and-promote cycle, per `docs/icm/convention.md` §2 (the
draft/admitted split governs *new or substantially rewritten* content;
adding a required section to an already-admitted stage's `CONTEXT.md` is
neither). Per ADR 0013 decision 6, only the promotable form of this
change (once actually made) needs independent review before it lands —
this adjudication record itself, being ICM-R2 pilot evidence, needs the
pilot's own reviewer step (`reference/proposal-icm-r-procedure-authority.md`
§8.11) before its findings are treated as settled.

## Alternatives considered

- **REHOME the directly-invoked entry (`00-check-scope`/`10-do-the-work`)
  to a Captain skill**, on the theory that "carry out a described task"
  looks conversational. Rejected: both stages already pass the
  execution-surface test (`convention.md` §2a) — they run as fresh,
  durable, stage-bound executions with declared Inputs/outputs, not live
  dialogue about what Work should exist; N1 adjudication A5 already
  litigated and rejected dissolving them once, for the same reason.
- **Treat the push/pr/ci gap as an engine-gap (PL-7) claim.** Rejected:
  nothing about the gap requires the runtime to own a new durable fact —
  it requires this workflow's own content to state an authority boundary
  it currently omits. Lower rungs (a stage-local J-clause) have not been
  attempted yet, so PL-7 is unreached per the ladder's own first-honest-
  rung rule (proposal §4.8).
- **Silently add a `--skip push,pr,ci` instruction to `30-start-run` on
  this producer's own authority**, resolving the gap rather than just
  recording it. Rejected per this Work's explicit brief: the underlying
  policy question is live and belongs to the owner, and per
  `bounded-judgment.md`'s own J0 procedure, a producer at J0 states the
  gap and a recommendation is optional evidence, not a substitute for the
  owner's decision — inventing the answer here would be exactly the
  "guess instead of ask" failure the ladder exists to prevent.
- **Leave `route-review-findings` as a `@@`-style reference and treat it
  as a future PL-6 shared helper once built.** Rejected as the current
  disposition: `convention.md` §4 rule 1 is explicit that `@@name` is
  context composition, not workflow composition, and a reference used to
  imply "run that other procedure" when no such procedure currently
  exists is a scope violation independent of whether it might exist
  later; the correct current statement is that the routing behavior is
  already folded in-package (BU-VAS-09), with a future-tense note if the
  CLI-SURFACE verb is ever built.

## Final disposition
STAND

## Validation evidence

- Source-valid: every existing behavior-unit citation in this package's
  seven stage `CONTEXT.md` files was read in full and traced to its
  already-archived N1 provenance (`docs/gauntlet/promoted-provenance/
  validate-and-ship.md`); no new citation was fabricated for this pass.
- Placement-valid: every stage's already-recorded PL-5 rung (`actor-stage
  (§6.4, judgment)`) was independently re-derived from the Placement
  Ladder in this pass and confirmed, not merely copied from the
  package's own table.
- Authority-valid: **not yet** — this is precisely what BU-VAS-13/14/15
  found missing. The package cannot be called authority-valid
  (`reference/proposal-icm-r-procedure-authority.md` §9.1 claim 3) until
  the four remediation items under "Surviving package design" land and
  the BU-VAS-15 gap is either ruled on or explicitly deferred by the
  owner with a citable record.
- Structurally valid: all seven stage directories, their `output/
  README.md` declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly, not assumed.
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the
  package; `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims (needs_input on a real/scripted J0 case,
  operation without Captain present) remain to be measured separately.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7); it
  does not self-promote.
