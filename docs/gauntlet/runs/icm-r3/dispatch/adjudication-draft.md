# Package adjudication: dispatch

ICM-R3 full-reconciliation pass, `docs/adr/0013-icm-r0-owner-rulings.md`;
method per `reference/proposal-icm-r-procedure-authority.md` §8 (§10.4
scope: all 23 published workflows, run in bounded waves by delegation
cluster); record shape per `docs/icm/record-shapes.md` §6. Producer pass
only — independent review is a separate step (§8.11 of the proposal;
§6.2/6.3 of `docs/icm/convention.md`) and has not run yet. This record is
itself draft — it does not self-promote (ADR 0013 decisions 6-7).

Package-specific note from the dispatch brief: check whether `dispatch`'s
delegation citations to `drain-fleet` and `respond-to-worker` carry the
same class of dangling-reference defect ICM-R2 found in `validate-and-ship`
(`BU-VAS-10`) and `research` (`B9`). Confirmed below (`BU-DISP-11`,
`BU-DISP-12`) against the *current* content of both cited packages, not
assumed from the hint. `cross-repo-work`'s `50-handoff-or-stop` delegates
to `dispatch` (`.sergeant/workflows/cross-repo-work/CONTEXT.md` line 30),
so this record's classification of `dispatch` itself is load-bearing for
that wave-2 package and should be adjudicated first, per the brief.

## Original intention

Given a project, a brief or tracked task, and a repository set, produce one
durable task with an isolated work surface, a rendered mission brief, and a
running agent per repository — with every side effect validated and gated
before the next repository's dispatch begins
(`.sergeant/workflows/dispatch/CONTEXT.md` "Purpose"; `index.md`
description). Promoted into the N1 reference corpus as candidate **W8**
`dispatch` (`docs/gauntlet/contracts/N1.md`,
`docs/icm/promotion-spec-2026-08-11.md`), with the full behavior-unit
citation trail and N1 adjudication A3/A4 fold record already archived at
`docs/gauntlet/promoted-provenance/dispatch.md`. This ICM-R3 pass does not
re-run that N1 extraction; it applies the Placement and Bounded-Judgment
ladders on top of the already-cited N1 content and checks the package's
compliance with ADR 0013's rulings, including whether its two named
cross-workflow delegations still resolve.

The package's own `CONTEXT.md` already documents that N1 adjudication A4
folded six of twelve originally-extracted stages into their nearest
judgment-bearing neighbor (stage count 12 → 6) after applying the §6.3
reimplementation test package-internally. This pass re-derives that same
test independently rather than trusting the package's self-report, per
`reference/proposal-icm-r-procedure-authority.md` §8.9 ("Self-check is
necessary but not promotion authority") applied one level up: a producer
does not inherit a prior producer's self-check as settled classification.

## Current trigger and outcome

One linear stage list (`workflow.toml`: `00-check-queue-and-plan`,
`05-classify-risk`, `15-check-admission`, `20-prepare-intent`,
`80-monitor`, `90-reconcile-fleet`), single entry at `00-check-queue-and-plan`.

Trigger (all six stage `CONTEXT.md` files, uniformly): work spans
repositories, contains two or more independent repository-owned tasks,
needs an isolated review worker, or the user asks for workers.

Outcome: one durable task with an isolated work surface, a rendered
mission brief, and a running agent per targeted repository, with every
side effect (tracked-work creation, worktree acquisition, worker-process
launch) validated and gated before the next repository's dispatch begins,
followed by escalation handling without inference and a final per-repo
reconciliation gate that never completes merely because PRs exist.

## Driver and admission boundary

Driver: **stage actor**, throughout. Admission boundary: **in-Work** — the
package receives an already-defined objective, repository set, and
(optionally) a tracked-task reference; it does not itself decide *whether*
work should exist, only how to execute an already-scoped multi-repository
objective. This matches the execution-surface test
(`docs/icm/convention.md` §2a): "would a human type `sgt run '<intent>'
--workflow dispatch`?" — yes, given an already-defined objective and repo
set. The package's own stage table already labels every stage
"actor-stage (§6.4, judgment)", which this pass independently re-derives
below rather than accepting at face value.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-DISP-01 | `00-check-queue-and-plan/CONTEXT.md` — reuse an existing td task's brief/branch/context when one covers the work; otherwise confirm a free-form brief plus explicit repo list as accurate before anything is created | PL-5 | J2 (delegated: task-vs-brief mode determination, `BU-P5-057`/`058`/`059`/`060`; plan confirmation is a J4-compatible check against the user's own stated request, not a new decision) | STAND | `00-check-queue-and-plan` |
| BU-DISP-02 | `05-classify-risk/CONTEXT.md` — an objective matching a fixed safety-sensitive/stateful keyword set cannot use the standard-isolated intent path and must be given an explicit `--intent-file` | PL-5 | J5 (governing: the fixed keyword set is a stage-contract prohibition on the lightweight path, `BU-P6-048`, `BU-P7-016`, `BU-P8-069`) with residual J2 for judging whether objective text implies a listed risk category not verbatim-matched | STAND | `05-classify-risk` |
| BU-DISP-03 | `15-check-admission/CONTEXT.md` Helper section — harness/model-tuple/identity validated and rejected before any durable state exists (folded `10-preflight-capabilities`, N1 adjudication A4) | PL-6 | J5 (governing: every probe fails closed before any fleet task directory, intent file, or worktree is created — `BU-P1-057`/`058`/`060`/`093`/`094`, `BU-P6-107`/`124`, `BU-P7-002`/`072`/`073`/`078`) | STAND — the fold itself is independently confirmed: swapping which probe implements harness/model/identity validation leaves this stage's admission checkpoint unchanged, which is exactly §6.3's helper test | `15-check-admission` (helper) |
| BU-DISP-04 | `15-check-admission/CONTEXT.md` — the fleet-wide admission lock is held only across the first durable side effect (tracked-work creation, when no task reference was supplied), then released, so dispatch never holds a shared lock across its own much longer worktree/launch sequence | PL-5 | J2 (delegated: this is the workflow's real judgment-bearing checkpoint — deciding the lock's narrow hold window against the specific side effect it protects, `BU-P6-128`) | STAND, **with a required correction to how this checkpoint's own justification is stated** — see "The drain-fleet reference" below | `15-check-admission` |
| BU-DISP-05 | `20-prepare-intent/CONTEXT.md` — one canonical `.sergeant-intent.md` revision is written identically to fleet state and every selected work surface, treated as canonical by every downstream actor (implementer, reviewer, recovery, final validation) | PL-5 | J5 (governing: canonical-revision identity across fleet state and every worktree is a correctness invariant, not a local choice — `BU-P8-059`) with J1 for the mechanical order in which the identical copies are written | STAND | `20-prepare-intent` |
| BU-DISP-06 | `80-monitor/CONTEXT.md` Helper section, item 1 — bulk fleet reconciliation (syncs worktree status, stops only identity-verified done/failed processes, bounded grace period, preserves needs_input/blocked/orphaned) runs automatically before new work is created (folded `40-reconcile-before-launch`; ordering fixed by N1 adjudication A3/BH-01) | PL-6 | J5 (governing: the grace-period bound and the never-sweep-needs_input/blocked/orphaned rule are fixed, `BU-P8-070`) | STAND | `80-monitor` (helper) |
| BU-DISP-07 | `80-monitor/CONTEXT.md` Helper section, item 2 — all-or-nothing tracked-work creation across every target repo, rolled back on any failure (folded `30-create-tracked-work`) | PL-6 | J5 (governing: partial task-set creation is never left standing — `BU-P5-088`, `BU-P6-036`) | STAND | `80-monitor` (helper) |
| BU-DISP-08 | `80-monitor/CONTEXT.md` Helper section, item 3 — isolated work surface per repo at a deterministic sibling path, treehouse-pool preference, refusal to silently discard unpushed committed work absent explicit `--adopt-branch` (folded `50-acquire-surface`) | PL-6 | J5 (governing: the unpushed-work guard and its non-destructive `--adopt-branch` escape are fixed safety behavior — `BU-P6-125`, `BU-P7-069`/`070`) | STAND | `80-monitor` (helper) |
| BU-DISP-09 | `80-monitor/CONTEXT.md` Helper section, item 4 — mission/instructions/dependency-notes/delivery-requirements durably rendered into the worker's starting brief, including a verbatim explicit override, before the worker begins (folded `60-render-brief`) | PL-6 | J5 (governing: instruction merge order defaults→group→repo, with an explicit override always winning — `BU-P7-112`) | STAND | `80-monitor` (helper) |
| BU-DISP-10 | `80-monitor/CONTEXT.md` Helper section, item 5 — launch evidence written `intended` then promoted to `confirmed` only on observed readiness; every per-repo launch failure records an orphaned status with a diagnostic before the loop aborts (folded `70-launch-and-record`) | PL-6 | J5 (governing: the two-state intent/confirmed distinction and the per-repo fail-loud requirement are fixed — `BU-P6-110`/`126`, `BU-P7-111`) | STAND | `80-monitor` (helper) |
| BU-DISP-11 | `80-monitor/CONTEXT.md` — once workers are running, escalations are read in full and a human decision obtained without inference, then delivered to the exact task/repo pair | PL-5 | J2 (delegated: interpreting an escalation's evidence and options, `BU-P5-066`/`067`/`068`) with an explicit **J0** carve-out — "without inference" is itself the stage contract's own prohibition on the actor guessing consequential intent on the human's behalf, the same shape as `validate-and-ship/40-drive-gates`'s ask-user carve-out (proposal §3.4) | STAND, **with a required correction to how delivery is described** — see "The respond-to-worker reference" below | `80-monitor` |
| BU-DISP-12 | `90-reconcile-fleet/CONTEXT.md` — per-repo verification of pinned scope, validation, review artifacts, zero blocking findings, CI, and resolved review threads, plus dependency merge order; never complete merely because PRs exist | PL-5 | J2 (delegated: itemized gate verification per repo, `BU-P5-070`, `BU-P1-006`) with J5 (governing: PR existence alone never triggers reconciliation, `BU-P5-071`) | STAND | `90-reconcile-fleet` |
| BU-DISP-13 | All six stage `CONTEXT.md` files — uniform `## Judgment required` boilerplate paragraph; no stage names its J2 delegations, J1 local choices, or J0 escalation triggers in the required shape | N/A (authoring-format compliance, not a placement question) | J5 (`docs/icm/convention.md` §6.1 / ADR 0013 decision 4: every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section, "always... omission is never ambiguous" — a governing requirement this package predates and does not yet satisfy) | STAND (package identity correct; in-place content amendment required — see Surviving package design) | all six stage `CONTEXT.md` files |
| BU-DISP-14 | `CONTEXT.md` (L1) — no `## Authority envelope` section exists | N/A | J5 (`convention.md` §6.1: every workflow Layer-1 `CONTEXT.md` carries an `## Authority envelope` section) | STAND, in-place amendment required | `CONTEXT.md` |
| BU-DISP-15 | `15-check-admission/CONTEXT.md` "Delegation" — "this stage's outcome is produced by running **drain-fleet** to its own completion" | none resolvable as written — **dangling reference to a retired, never-built package** | — | **FOLD** (correct the reference in place; no placement change to this package) | `15-check-admission/CONTEXT.md`, `CONTEXT.md` |
| BU-DISP-16 | `80-monitor/CONTEXT.md` "Delegation" — "this stage's outcome is produced by running **respond-to-worker** to its own completion" | none resolvable as written — **dangling reference to a retired, absorbed package** | — | **FOLD** (correct the reference in place; no placement change to this package) | `80-monitor/CONTEXT.md`, `CONTEXT.md` |

## The `drain-fleet` reference (BU-DISP-15, full record)

`15-check-admission/CONTEXT.md` states the stage's checkpoint "is
judgment-bearing only insofar as it depends on `drain-fleet`'s
admission-block state" and its "Additional note" argues this is "a real
cross-workflow dependency (this stage's outcome is produced by running an
entire other workflow to its own completion, not by swapping a local
implementation detail), which is why it survives N1 adjudication A4's
§6.3 reimplementation test."

`drain-fleet` does not exist as a package. It is not under
`.sergeant/workflows/` (confirmed against the current directory listing
and `.sergeant/index.md`'s 20-package catalog) or
`.sergeant/drafts/workflows/`. It was retired at the 2026-08-12 re-homing
pass: "CLI-SURFACE, NET-NEW-SURFACE — no admission-block primitive exists;
engine-gap **G4**" (`docs/icm/re-homing-record-2026-08-12.md` line 28).
The narrower one-owner case is covered by the already-shipped `sgt daemon
stop` (MVP-3); the broader multi-actor fleet-wide drain/force-stop
`drain-fleet` described stays an open, unbuilt engine gap. No package or
CLI verb named `drain-fleet` runs today.

This means `15-check-admission`'s own stated justification for surviving
A4's fold sweep — "running drain-fleet to completion" — is not literally
true: there is no other workflow this stage invokes or waits on. Re-deriving
the checkpoint independently of that claim: the behavior this stage
actually performs is acquiring a fleet-wide admission lock across exactly
one durable side effect and releasing it immediately after (`BU-P6-128`).
That is a real, self-contained judgment-bearing checkpoint on its own
terms — narrow-lock-then-release is a choice this stage makes about its
own execution, not a delegation to another procedure — so **BU-DISP-04
still survives at PL-5 independent of whether `drain-fleet` exists**. What
does not survive is the *reason given* for why it survives: the correct
statement is that this stage holds a lock a *future* fleet-wide drain
operation (engine-gap G4, unbuilt) would also need to respect, not that
this stage runs `drain-fleet` to completion today. The current text
overclaims a live cross-workflow dependency that does not exist, which is
exactly the class of defect the brief asked this pass to check for
(`BU-VAS-10`, `research`'s `B9`).

**Rungs checked:** J5 — no governing constraint requires this stage to
literally invoke another workflow; the lock-then-release rule is this
stage's own contract. J4/J3 — not applicable, no user or settled-record
question is at stake. J2 — this producer is not authorized to rewrite the
package's live content (draft-and-rehome step, ADR 0013 decision 6,
applies to REHOME/SPLIT/HARVEST verdicts, not to STAND); the correction is
recorded as a required remediation, not applied here.

## The `respond-to-worker` reference (BU-DISP-16, full record)

`80-monitor/CONTEXT.md` states this stage's outcome "is produced by
running **respond-to-worker** to its own completion." `respond-to-worker`
does not exist as a package. It was retired at the same 2026-08-12
re-homing pass: "CLI-SURFACE, ABSORBED (retriage pass 1 + absorbed-sweep)
— 'collides with shipped `sgt respond` (`src/cli.rs:89`)'; its own
`00-precondition-check`'s only judgment was idempotency filtering,
`40-apply-and-acknowledge` applied an already-made human decision. Nowhere
new — the shipped `sgt respond` / `POST /v1/work/{id}/input` already is
this" (`docs/icm/re-homing-record-2026-08-12.md` line 22). This package's
own `index.md` already acknowledges the general risk this reference falls
into — "context composition today, not true nested-workflow invocation" —
but understates the specific defect: the cited target is not merely
un-invocable as a nested workflow, it is not a workflow at all any more.

Re-deriving `80-monitor`'s escalation-delivery checkpoint independently:
the behavior described (read the full escalation, obtain an explicit human
decision without inferring consequential intent, deliver it to the exact
task/repo pair, `BU-P5-066`/`067`/`068`) is real judgment this stage
performs itself; the *delivery mechanism* for that decision is the already
-shipped `sgt respond` command / `POST /v1/work/{id}/input` — the same
surface that absorbed `respond-to-worker` outright. **BU-DISP-11 survives
at PL-5** on the same terms as `BU-DISP-04` above: the checkpoint's
judgment content is real and self-contained; only the stated mechanism for
carrying out the delivery half of it is wrong as written, and should name
the shipped CLI/API surface instead of a workflow that no longer exists.

**Rungs checked:** same shape as the `drain-fleet` record above — J5/J4/J3
not applicable; J2 correction recorded, not applied by this producer.

## Surviving package design

No stage moves, merges, splits, or renames. The six-stage linear sequence
and every already-cited N1 behavior unit remain correctly placed at PL-4
(package) / PL-5 (each of the four judgment-bearing stages:
`00-check-queue-and-plan`, `05-classify-risk`, `15-check-admission`,
`20-prepare-intent`, `80-monitor`, `90-reconcile-fleet` — all six carry
real judgment, none is itself only deterministic machinery) / PL-6 (each
identified folded helper). N1 adjudication A4's fold from twelve extracted
stages to six is independently re-confirmed by this pass, not merely
copied. The package requires **in-place content amendment**, not
restructuring:

1. Add a `## Bounded judgment` section (per `convention.md` §7.3 /
   `bounded-judgment.md`) to each of the six stage `CONTEXT.md` files,
   replacing or supplementing the current `## Judgment required`
   boilerplate with named J2 delegations, J1 local choices, and J0
   escalation triggers specific to that stage — most of this is a direct
   restatement of judgment content this package's Behavior contract
   sections already carry informally (see the J boundary column above).
   `80-monitor`'s section in particular needs an explicit J0 clause for
   the escalation-without-inference rule (`BU-DISP-11`), mirroring
   `validate-and-ship/40-drive-gates`'s ask-user carve-out.
2. Add a `## Authority envelope` section to the workflow-level
   `CONTEXT.md` (per `convention.md` §7.2).
3. Correct the dangling `drain-fleet` reference in
   `15-check-admission/CONTEXT.md` (its "Purpose", "Additional note", and
   "Delegation" sections) and in `CONTEXT.md`'s "Relationships to other
   workflows": state that this stage itself acquires and releases the
   fleet-wide admission lock across exactly one durable side effect; note
   that a future fleet-wide drain/force-stop operation (engine-gap **G4**,
   `docs/icm/re-homing-record-2026-08-12.md` line 28, currently unbuilt)
   would need to respect the same lock, rather than claiming this stage
   currently runs `drain-fleet` to completion.
4. Correct the dangling `respond-to-worker` reference in
   `80-monitor/CONTEXT.md`'s "Delegation" section and in `CONTEXT.md`'s
   "Relationships to other workflows": state that escalation responses are
   delivered via the shipped `sgt respond` command / `POST
   /v1/work/{id}/input` (`docs/icm/re-homing-record-2026-08-12.md` line
   22), not by delegating to a `respond-to-worker` workflow.
5. `index.md`'s "Both targets are published in this library" line (line
   28) is also false as written for both cited targets and needs the same
   correction as items 3-4 above.

None of these five amendments changes which package owns the behavior, so
none triggers ADR 0013's REHOME/SPLIT/HARVEST draft-and-rehome step
(decision 6; task brief). They are recorded here as the concrete
remediation this adjudication found, for the owner/reviewer to schedule —
this producer does not apply them to the live package (§8.9's self-check
discipline: a producer records findings, an independent step or explicit
human gate accepts and lands them).

## Inputs and outputs

Inputs: as declared in each stage's own Inputs table (all six already
comply with `record-shapes.md` §1a — verified during Inventory: each names
exactly the prior stage's `output/README.md`, in stage order, with no
forward reference). No contract-bearing dependency was found undeclared.
The `drain-fleet` and `respond-to-worker` references are prose delegation
in "Delegation"/"Relationships to other workflows" sections, not declared
Inputs-table entries — exactly the kind of unresolved reference §1a rule 1
asks a reviewer to catch, and exactly the shape `BU-VAS-10` found in
`validate-and-ship`.

Outputs: `output/README.md` in each of the six stages declares its
expected artifact and disposition. Five of six (`00-check-queue-and-plan`,
`05-classify-risk`, `15-check-admission`, `20-prepare-intent`,
`80-monitor`) are `evidence` (Work-branch record only, including
`80-monitor`'s own note that "each of the five folded stages' own output
declared `evidence` disposition before A4; the merged record keeps that
disposition"); `90-reconcile-fleet`'s is `promote` (workflow deliverable),
correctly reflecting that it is the terminal stage. No violation found in
the Layer 4 declarations.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (not a draft) — its structural and provenance
identity does not change. The five remediation items above are ordinary
content edits to an admitted workflow and should go through this
repository's normal review path for workflow content changes, not a new
draft-and-promote cycle, per `docs/icm/convention.md` §2 (the
draft/admitted split governs *new or substantially rewritten* content;
correcting a dangling reference or adding a required section to an
already-admitted stage's `CONTEXT.md` is neither). Per ADR 0013 decision
6, only the promotable form of this change (once actually made) needs
independent review before it lands — this adjudication record itself,
being ICM-R3 evidence, needs the reconciliation's own reviewer step
(`reference/proposal-icm-r-procedure-authority.md` §8.11) before its
findings are treated as settled.

Because `cross-repo-work/50-handoff-or-stop` delegates to `dispatch`
(`.sergeant/workflows/cross-repo-work/CONTEXT.md` line 30), this record's
STAND verdict is load-bearing evidence for `cross-repo-work`'s own
ICM-R3 pass: `dispatch`'s package identity and stage names are unchanged
by this adjudication, so `cross-repo-work`'s delegation citation does not
itself become a new dangling reference as a side effect of this pass.

## Alternatives considered

- **Treat `15-check-admission` and `80-monitor` as further foldable into
  their neighbors**, on the theory that once their delegation targets are
  known not to exist, the stages lose the "real cross-workflow dependency"
  argument A4 used to keep them separate. Rejected: re-deriving each
  checkpoint's judgment content independently of the broken delegation
  claim (see the two full records above) shows real, self-contained
  judgment survives in both cases — the lock-acquire-then-release decision
  in `15-check-admission` and the escalation-without-inference decision in
  `80-monitor`. The defect is in how the justification is *stated*, not in
  whether the checkpoint is real.
- **Treat the `drain-fleet`/`respond-to-worker` references as REHOME
  triggers for this package**, on the theory that a package citing two
  non-existent delegation targets is itself unsound as published.
  Rejected: the package's own stage boundaries, judgment content, and
  outcome do not depend on those targets actually being separate
  workflows — only the prose describing *how* two of six checkpoints are
  achieved is wrong. Correcting prose in place is `convention.md` §2's
  normal content-edit path, not grounds to move or restructure the
  package (mirrors `BU-VAS-10`'s FOLD disposition exactly).
- **Silently rewrite the two dangling references on this producer's own
  authority**, since the correct replacement text is evident from already
  -published re-homing records. Rejected per this Work's brief ("Produce
  the files and stop — you are the producer, not the reviewer") and per
  `reference/proposal-icm-r-procedure-authority.md` §8.9/§4.9: a producer
  does not self-promote its own output; the remediation is recorded here
  for the independent review/reconcile step to land.
- **Treat `05-classify-risk`'s fixed keyword set as PL-6 deterministic
  machinery** rather than PL-5 judgment, since keyword matching sounds
  mechanical. Rejected: the stage still requires the actor to read the
  objective's actual text and judge whether it falls under a listed risk
  category (not always a literal substring match — e.g. "stateful" is
  broader than any single keyword), which is exactly the class of
  judgment §6.4 describes; the fixed keyword *list itself* is the J5
  governing constraint, but applying it to free text is not mechanical.

## Final disposition
STAND

## Validation evidence

- Source-valid: every existing behavior-unit citation in this package's
  six stage `CONTEXT.md` files was read in full and traced to its already
  -archived N1 provenance (`docs/gauntlet/promoted-provenance/
  dispatch.md`); no new citation was fabricated for this pass.
- Placement-valid: every stage's already-recorded PL-5 rung
  ("actor-stage (§6.4, judgment)") was independently re-derived from the
  Placement Ladder in this pass and confirmed, not merely copied from the
  package's own table; N1 adjudication A4's twelve-to-six fold was
  independently re-checked against §6.3's reimplementation test rather
  than accepted on the package's self-report.
- Authority-valid: **not yet** — this is precisely what `BU-DISP-13`/`14`
  found missing. The package cannot be called authority-valid
  (`reference/proposal-icm-r-procedure-authority.md` §9.1 claim 3) until
  the five remediation items under "Surviving package design" land.
- Delegation-valid (this pass's specific check, per the task brief): both
  of this package's cited cross-workflow delegations (`drain-fleet`,
  `80-monitor` → `respond-to-worker`) were independently verified against
  the *current* `.sergeant/workflows/` directory listing, `.sergeant/
  index.md`'s 20-package catalog, and `docs/icm/re-homing-record-
  2026-08-12.md` — neither target exists as a live package or draft.
  Both are dangling references requiring correction (`BU-DISP-15`,
  `BU-DISP-16`), the same defect class as `validate-and-ship`'s
  `BU-VAS-10` and `research`'s `B9`. Both underlying checkpoints
  (`BU-DISP-04`, `BU-DISP-11`) were re-derived independently of the broken
  claim and confirmed to carry real, self-contained judgment regardless.
- Structurally valid: all six stage directories, their `output/
  README.md` declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly, not assumed.
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the package;
  `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims (needs_input on a real/scripted J0 case,
  operation without Captain present) remain to be measured separately.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7); it
  does not self-promote.
