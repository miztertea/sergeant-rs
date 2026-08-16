# Package adjudication: cross-repo-work

ICM-R3 full-reconciliation pass, `docs/adr/0013-icm-r0-owner-rulings.md`;
method per `reference/proposal-icm-r-procedure-authority.md` §8 (§10.4
scope: all 23 published workflows, run in bounded waves by delegation
cluster). Producer pass only — independent review is a separate step
(§8.11 of the proposal; §6.2/6.3 of `docs/icm/convention.md`) and has not
run yet. This record is itself draft — it does not self-promote (ADR 0013
decisions 6-7).

**Wave 2, depends on wave 1's `dispatch` verdict.** `50-handoff-or-stop`
delegates to `dispatch`; `dispatch`'s ICM-R3 pass
(`docs/gauntlet/runs/icm-r3/dispatch/adjudication-draft.md`, confirmed by
its `review.md`) reached **STAND** with no rename, no restructuring, and no
placement change to any of its six stages. That verdict is load-bearing
here: `cross-repo-work`'s delegation citation to `dispatch` names a live,
unchanged package. `dispatch`'s reviewer pass additionally flagged, as an
item explicitly out of its own scope, that `cross-repo-work/60-reconcile/
CONTEXT.md` names `reconcile-and-cleanup-fleet` as an adjacent owned
procedure and that name is absent from the current catalog — "noted here
only so `cross-repo-work`'s own ICM-R3 pass does not miss it." That flag is
independently re-verified below (`BU-CRW-07`), not assumed from the hint.

`60-reconcile` also merely *names* (does not invoke) `dispatch`'s fleet
reconciliation and `reconcile-and-cleanup-fleet`'s cleanup as adjacent,
owned procedures, per this package's own `CONTEXT.md` line 31. That
citation's accuracy is checked directly below: the `dispatch` half is
confirmed; the `reconcile-and-cleanup-fleet` half is not.

## Original intention

Given a requested outcome that spans more than one repository, decompose
it so every required behavior has exactly one owning repository, an
acyclic dependency position, a brief, and acceptance evidence — before any
dispatch happens (`.sergeant/workflows/cross-repo-work/CONTEXT.md`
"Purpose"; `index.md` description). Promoted into the N1 reference corpus
as candidate **W7** `cross-repo-work`
(`docs/gauntlet/contracts/N1.md`, `docs/icm/promotion-spec-2026-08-11.md`),
with the full behavior-unit citation trail archived at
`docs/gauntlet/promoted-provenance/cross-repo-work.md`. This ICM-R3 pass
does not re-run that N1 extraction; it applies the Placement and
Bounded-Judgment ladders on top of the already-cited N1 content and checks
the package's compliance with ADR 0013's rulings, including whether its
two named cross-workflow references (`dispatch`, `reconcile-and-cleanup
-fleet`) still resolve — the same delegation-fidelity check the task brief
asked `dispatch`'s pass to run on its own two references, applied here to
this package's own two.

The package's own `CONTEXT.md` already documents that N1 adjudication A4
folded `30-inspect-repository-state` into `40-define-delivery-gates` as a
preceding helper invocation (stage count 6 extracted → 5 surviving) after
applying the §6.3 reimplementation test. This pass re-derives that same
test independently rather than trusting the package's self-report, per
`reference/proposal-icm-r-procedure-authority.md` §8.9 ("Self-check is
necessary but not promotion authority") applied one level up, the same
discipline `dispatch`'s own ICM-R3 pass applied to its A4 fold.

## Current trigger and outcome

One linear stage list (`workflow.toml`: `10-assign-ownership`,
`20-define-dependency-order`, `40-define-delivery-gates`,
`50-handoff-or-stop`, `60-reconcile`), single entry at
`10-assign-ownership`. Directory listing (`10-`, `20-`, `40-`, `50-`,
`60-`) agrees with `workflow.toml`'s declared order
(`docs/icm/convention.md` §1 rule 4) — verified directly.

Trigger (all five stage `CONTEXT.md` files, uniformly, and workflow-level
`CONTEXT.md`): resolved project context shows more than one repository
owns the requested outcome (not merely that the project has several
repos).

Outcome: a plan in which every required behavior has exactly one owning
repository, an acyclic dependency position, a brief, and acceptance
evidence; then either the plan is returned (planning-only) or control
passes to `dispatch`, with the coordinator never editing several
repositories itself; then, once dispatch has run, a reconciliation of PR
URLs, heads, CI, review threads, merge and deployment order, and terminal
task/fleet state — scoped strictly to the repositories this plan named.

## Driver and admission boundary

Driver: **stage actor**, throughout. Admission boundary: **in-Work** — the
package receives an already-defined multi-repository objective; it does
not itself decide *whether* work should exist, only how to decompose an
already-scoped cross-repository objective into an ownership/dependency/
delivery plan. This matches the execution-surface test
(`docs/icm/convention.md` §2a): "would a human type `sgt run '<intent>'
--workflow cross-repo-work`?" — yes, given an already-defined
multi-repository objective. The package's own stage table already labels
every surviving stage "actor-stage (§6.4, judgment)", which this pass
independently re-derives below rather than accepting at face value.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-CRW-01 | `10-assign-ownership/CONTEXT.md` — for each required behavior, name exactly one owning repository (only when it must change or produce delivery evidence) with role/deliverable/acceptance in a fixed, comparable shape; ambiguous ownership is resolved from the project graph and existing contracts first, the user is asked only when two repositories could legitimately own a user-visible or durable contract (`BU-P5-041`/`042`/`043`) | PL-5 | J2 (delegated: repository-ownership assignment against the project graph and existing contracts, named bounds) with an explicit ask-user carve-out — "the user is asked only when two repositories could legitimately own" a durable contract is the stage's own narrow J0 trigger, not general license to ask about every ambiguity | STAND | `10-assign-ownership` |
| BU-CRW-02 | `20-define-dependency-order/CONTEXT.md` — dependency edges are created only when one repository's merged or deployed result is genuinely required by another, drawn from a small principled evidence vocabulary (contract/schema precedence, infra-before-runtime, independent-parallel-once-contract-approved, deploy dependency tracked separately from merge dependency); cycles are rejected before dispatch, and a genuinely coupled cycle is broken by defining the contract artifact or compatibility phase instead (`BU-P5-044`/`045`/`046`) | PL-5 | J2 (delegated: which evidence justifies an edge, and how to break a genuine cycle, `BU-P5-044`/`045`) with J5 (governing: no cyclic dependency graph ever reaches dispatch, `BU-P5-046` — a fixed prohibition, not a class of decision the stage weighs alternatives on) | STAND | `20-define-dependency-order` |
| BU-CRW-03 | `40-define-delivery-gates/CONTEXT.md` Helper invocations (formerly `30-inspect-repository-state`, folded by N1 adjudication A4) — non-main branches, uncommitted changes, ahead/behind state, active worktrees, and preserved workers are recorded for every owning repository without mutating anything; planning never stashes, resets, switches, or cleans repository state, and instead either routes existing canonical branch/worktree state into the worker brief or stops for a decision when state conflicts with the requested outcome (`BU-P5-047`/`048`) | PL-6 | J5 (governing: strictly read-only with respect to repository state — no stash/reset/switch/clean under any circumstance, `BU-P5-048`) | STAND — the fold itself is independently confirmed: swapping which command implements branch/worktree/ahead-behind inspection leaves this stage's checkpoint (a read-only record of repo state) unchanged, which is exactly §6.3's helper test, matching N1 adjudication A4's own reasoning re-derived rather than copied | `40-define-delivery-gates` (helper) |
| BU-CRW-04 | `40-define-delivery-gates/CONTEXT.md` — every per-repository delivery gate includes the owning td task (or its creation requirement), fixed point and preserved source state, repo-specific test/lint/typecheck/build commands, Standards and Spec review sources, PR dependency and deployment order, and any already-approved or still-missing data/security/destructive decisions; the plan is complete only once every owning repository has one implementation brief, acceptance evidence, and an acyclic dependency position (`BU-P5-049`/`050`) | PL-5 | J2 (delegated: defining each repository's concrete gate content from the inspected state and dependency graph, `BU-P5-049`) with J5 (governing: the plan's completion condition — brief + acceptance evidence + acyclic position for every owner — is fixed, not a judgment call, `BU-P5-050`) | STAND | `40-define-delivery-gates` |
| BU-CRW-05 | `50-handoff-or-stop/CONTEXT.md` — if the user requested planning only, the workflow stops after returning briefs, acceptance evidence, and dependency graph without dispatching or editing any repository; if implementation was requested, it hands off to `dispatch` via its launch command; the coordinating session never itself edits several repositories, and never itself performs `git checkout -b`, `git push -u origin`, or `gh pr create` as inline behavior — those belong to the dispatched worker (`BU-P5-051`, `BU-P7-017`) | PL-5, delegating at handoff to `dispatch` (PL-4, wave-1 STAND, unchanged identity) | J5 (governing: the coordinator never edits several repositories itself and never performs the worker's own git mutations inline — both are fixed prohibitions, not discretionary) with J2 residual (planning-only vs. implementation-requested determination from the user's actual request) | STAND, **delegation citation to `dispatch` confirmed accurate** — see "The `dispatch` reference" below | `50-handoff-or-stop` |
| BU-CRW-06 | `60-reconcile/CONTEXT.md` — after dispatched workers finish, reconcile PR URLs and final heads, required CI and unresolved review threads, merge order from dependency edges, deployment order and cross-repo release notes, and terminal task/fleet state for the repositories *this Work's plan named only*; the outcome is never reported complete until every owning repository has a terminal result or an explicit preserved blocker (`BU-P5-052`/`053`, scoped per N1 adjudication A8/BH-10) | PL-5 | J5 (governing: never report complete without a terminal-or-blocker fact for every named repo, and never assert facts about repositories, tasks, or fleet state outside this plan's own repo set, `BU-P5-053` plus the A8 scope note) with J2 residual (which reconciled fact — PR/CI/thread/merge-order/deploy state — applies to each repo) | STAND, **delegation/naming citation to `reconcile-and-cleanup-fleet` NOT accurate as written** — see "The `reconcile-and-cleanup-fleet` reference" below | `60-reconcile` |
| BU-CRW-07 | `CONTEXT.md` "Relationships to other workflows" (line 31) and `60-reconcile/CONTEXT.md` "Purpose"/"What must become true here"/"Scope note" — all name `reconcile-and-cleanup-fleet` as an "adjacent, owned procedure" alongside `dispatch` | none resolvable as written for the `reconcile-and-cleanup-fleet` half — **dangling reference to a retired, never-built package** | — | **FOLD** (correct the reference in place; no placement change to this package) | `CONTEXT.md`, `60-reconcile/CONTEXT.md` |
| BU-CRW-08 | All five stage `CONTEXT.md` files — uniform `## Judgment required` boilerplate paragraph; no stage names its J2 delegations, J1 local choices, or J0 escalation triggers in the required shape | N/A (authoring-format compliance, not a placement question) | J5 (`docs/icm/convention.md` §6.1 / ADR 0013 decision 4: every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section, "always... omission is never ambiguous" — a governing requirement this package predates and does not yet satisfy) | STAND (package identity correct; in-place content amendment required — see Surviving package design) | all five stage `CONTEXT.md` files |
| BU-CRW-09 | `CONTEXT.md` (L1) — no `## Authority envelope` section exists | N/A | J5 (`convention.md` §6.1: every workflow Layer-1 `CONTEXT.md` carries an `## Authority envelope` section) | STAND, in-place amendment required | `CONTEXT.md` |

Independently re-checked: unlike `dispatch/15-check-admission`, all five of
this package's stage `CONTEXT.md` files carry the `## Judgment required`
boilerplate uniformly (verified by direct heading read of each of the five
files, not assumed from the package's own description) — `BU-CRW-08`'s
"uniform" claim, unlike `dispatch`'s reviewed `BU-DISP-13`, is accurate as
written.

## The `dispatch` reference (BU-CRW-05, full record)

`50-handoff-or-stop/CONTEXT.md`'s "Delegation" section states: "This
stage's outcome is produced by running **dispatch** to its own completion
(context composition today — see `docs/icm/convention.md` §4 on `@@name`
versus true nested-workflow invocation, which does not exist yet)." This
is checked directly against the current `.sergeant/workflows/` directory
listing and `.sergeant/index.md`'s catalog: `dispatch` is present, `status:
published`, and its own ICM-R3 pass (`docs/gauntlet/runs/icm-r3/dispatch/
adjudication-draft.md`, reviewed and STAND-confirmed) changed no stage
name, no package identity, and no structural placement. `dispatch`'s
reviewer pass independently re-verified this same citation from
`dispatch`'s side ("`cross-repo-work` delegation citation... is live and
current") and found no false-pairing issue.

The parenthetical's own framing — "context composition today... true
nested-workflow invocation... does not exist yet" — is independently
correct and not the stale claim `dispatch`'s review found elsewhere
(`BU-DISP-03`'s "no `kind = \"execute\"` stage exists" defect): this
sentence is about nested-*workflow* invocation (a distinct engine
capability, tracked as open engine-gap **G6** in
`reference-corpus/engine-pressure.md`), not about `kind = "execute"`
stages existing at all. `repo-to-icm`'s `65-self-check` being a live
`execute` stage does not bear on whether a *child-workflow* invocation
primitive exists — it does not, and G6 records that gap accurately. No
correction needed here.

**Rungs checked:** J5 — no governing constraint requires this stage to
literally invoke `dispatch` as a nested workflow (none exists); the
hand-off is context composition, consistent with `convention.md` §4. J4/J3
— not applicable. J2 — this producer is not authorized to rewrite the
package's live content (draft-and-rehome step, ADR 0013 decision 6,
applies to REHOME/SPLIT/HARVEST verdicts, not to STAND); no correction is
needed here regardless, since the citation is accurate.

## The `reconcile-and-cleanup-fleet` reference (BU-CRW-06/07, full record)

`CONTEXT.md`'s "Relationships to other workflows" states: "`60-reconcile`
*names* (does not invoke) `dispatch`'s fleet reconciliation and
`reconcile-and-cleanup-fleet`'s cleanup as adjacent, owned procedures (N1
adjudication A8, BH-10)." `60-reconcile/CONTEXT.md`'s "Purpose" and "What
must become true here" repeat the pairing ("that is `dispatch`'s and
`reconcile-and-cleanup-fleet`'s owned territory"), and its "Scope note"
states "`reconcile-and-cleanup-fleet` owns the actual cleanup decision and
mutation once every repo is terminal."

`reconcile-and-cleanup-fleet` does not exist as a package. It is not under
`.sergeant/workflows/` (confirmed against the current 17-entry directory
listing and `.sergeant/index.md`'s catalog) or `.sergeant/drafts/
workflows/` (this working tree has no `drafts/` directory at all,
confirmed directly). It was retired at the 2026-08-12 re-homing pass:
"CLI-SURFACE, PARTIAL — per-repo surface teardown ABSORBED (`recovery.rs`'s
automatic reconciliation); multi-repo 'fleet task' grouping/handshake-ack
cleanup is NET-NEW, no such domain object exists" — candidate destination
"`sgt fleet cleanup` verb candidate **if** the fleet-grouping object is
ever ruled in (currently **NOT-EVER** per North Star's 'fleet as a domain
object' line)" (`docs/icm/re-homing-record-2026-08-12.md` line 25). This is
a materially different situation from `dispatch`'s two dangling references:
`drain-fleet` is an open, unbuilt engine-gap (**G4**) that could someday be
built, and `respond-to-worker` was absorbed into a command that ships
today (`sgt respond`). `reconcile-and-cleanup-fleet`'s multi-repo cleanup
half is not merely unbuilt — the object it would operate on ("fleet as a
domain object") is doctrinally foreclosed by the North Star as things
stand. Calling it an "owned procedure" overstates its status twice over:
it is not a live package, and the thing it would own is currently ruled
out from ever existing.

This is precisely the defect class `dispatch`'s reviewer pass flagged as
out of its own scope and asked this package's ICM-R3 pass to check
(`docs/gauntlet/runs/icm-r3/dispatch/review.md`, "Additional check" —
"noted here only so `cross-repo-work`'s own ICM-R3 pass does not miss
it"), and the same shape as `BU-DISP-15`/`16` and `research`'s `B9`.

Re-deriving `60-reconcile`'s checkpoint independently of the broken half of
the claim: the behavior this stage actually performs — reconciling PR
URLs, heads, CI, review threads, merge/deploy order, and terminal state
for exactly the repositories this Work's plan named, never asserting
anything about repos or fleet state outside that set (`BU-P5-052`/`053`,
A8 scope note) — is real, self-contained judgment that does not depend on
`reconcile-and-cleanup-fleet` existing. **`BU-CRW-06` survives at PL-5**
independent of the dangling reference, on the same terms `dispatch`'s
`BU-DISP-04`/`11` survived independent of its two broken references. What
does not survive is the *characterization* of `reconcile-and-cleanup-fleet`
as an adjacent owned procedure this stage's output feeds.

The `dispatch` half of the same sentence is accurate and requires no
change (confirmed above, "The `dispatch` reference"); only the
`reconcile-and-cleanup-fleet` half needs correction. `reference-corpus/
engine-pressure.md`'s own G6 entry (lines 419-440, read directly), which
this package's own `60-reconcile/CONTEXT.md` cites for the underlying
duplication pressure, is independently checked and does **not** repeat
this defect — it discusses only `dispatch`'s reconciliation sweep by name
and never characterizes `reconcile-and-cleanup-fleet` as a currently owned
procedure; no correction is needed there.

**Rungs checked:** J5 — no governing constraint requires this stage's
prose to name a non-existent package as an owned procedure; the correct
statement is that the underlying wish to compose with a real fleet-cleanup
procedure is recorded as evidence for engine-gap G6, not that such a
procedure currently exists and owns anything. J4/J3 — not applicable. J2 —
this producer is not authorized to rewrite the package's live content
(same boundary as the `dispatch` reference); the correction is recorded as
a required remediation, not applied here.

## Surviving package design

No stage moves, merges, splits, or renames. The five-stage linear sequence
and every already-cited N1 behavior unit remain correctly placed at PL-4
(package) / PL-5 (each of the four judgment-bearing stages:
`10-assign-ownership`, `20-define-dependency-order`,
`40-define-delivery-gates`, `50-handoff-or-stop`, `60-reconcile` — all five
carry real judgment) / PL-6 (the one folded helper, formerly
`30-inspect-repository-state`). N1 adjudication A4's fold from six
extracted stages to five is independently re-confirmed by this pass, not
merely copied. The package requires **in-place content amendment**, not
restructuring:

1. Add a `## Bounded judgment` section (per `convention.md` §7.3 /
   `bounded-judgment.md`) to each of the five stage `CONTEXT.md` files,
   replacing or supplementing the current `## Judgment required`
   boilerplate with named J2 delegations, J1 local choices, and J0
   escalation triggers specific to that stage — most of this is a direct
   restatement of judgment content this package's Behavior contract
   sections already carry informally (see the J boundary column above).
   `10-assign-ownership`'s section in particular needs an explicit J0
   clause for genuinely contested cross-repository ownership
   (`BU-CRW-01`), and `20-define-dependency-order`'s needs an explicit J5
   clause that no cyclic graph ever reaches dispatch (`BU-CRW-02`).
2. Add a `## Authority envelope` section to the workflow-level
   `CONTEXT.md` (per `convention.md` §7.2).
3. Correct the `reconcile-and-cleanup-fleet` half of the "Relationships to
   other workflows" line in `CONTEXT.md` and the "Purpose"/"What must
   become true here"/"Scope note" text in `60-reconcile/CONTEXT.md`: state
   that `reconcile-and-cleanup-fleet` is not a live package — its per-repo
   teardown half was absorbed into `recovery.rs`'s automatic
   reconciliation and its multi-repo fleet-grouping/cleanup half is
   doctrinally unbuilt (currently ruled out by the North Star's "fleet as
   a domain object" line, `docs/icm/re-homing-record-2026-08-12.md` line
   25) — and that the underlying wish for a real composed cleanup
   procedure is recorded as evidence for existing engine-gap **G6**
   (`reference-corpus/engine-pressure.md`), not as a currently owned
   procedure this stage's output feeds. The `dispatch` half of the same
   sentences is accurate and unchanged.

Neither amendment changes which package owns the behavior, so neither
triggers ADR 0013's REHOME/SPLIT/HARVEST draft-and-rehome step (decision 6;
task brief). They are recorded here as the concrete remediation this
adjudication found, for the owner/reviewer to schedule — this producer
does not apply them to the live package (§8.9's self-check discipline: a
producer records findings, an independent step or explicit human gate
accepts and lands them).

## Inputs and outputs

Inputs: as declared in each stage's own Inputs table (all five already
comply with `record-shapes.md` §1a — verified during Inventory: each names
exactly the prior stage's `output/README.md`, in stage order, with no
forward reference; `10-assign-ownership` correctly names only `../
CONTEXT.md` (L1) as its first-stage input). No contract-bearing dependency
was found undeclared. The `dispatch` and `reconcile-and-cleanup-fleet`
references are prose in "Delegation"/"Relationships to other
workflows"/"Purpose"/"Scope note" sections, not declared Inputs-table
entries — exactly the kind of unresolved reference §1a rule 1 asks a
reviewer to catch, and exactly the shape `BU-DISP-15`/`16` and `BU-VAS-10`
found in `dispatch` and `validate-and-ship`.

Outputs: `output/README.md` in each of the five stages declares its
expected artifact and disposition. Four of five
(`10-assign-ownership`, `20-define-dependency-order`,
`40-define-delivery-gates`, `50-handoff-or-stop`) are `evidence`
(Work-branch record only); `60-reconcile`'s is `promote` (workflow
deliverable), correctly reflecting that it is the terminal stage per
`workflow.toml`'s own stage order. `60-reconcile/output/README.md` itself
already flags, at its own promotion, that the workflow has no dedicated
finalize step despite declaring a `promote` output (`docs/icm/
convention.md` §1a open question 1) — independently re-confirmed: no
finalize stage or step exists anywhere in this package's five stages or
`workflow.toml`. This is recorded, not newly discovered by this pass; it
is the same open convention-level question (not a package-specific defect)
`convention.md` §1a already tracks as unresolved, and is not added to this
package's own remediation list for that reason.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (not a draft) — its structural and provenance
identity does not change. The three remediation items above are ordinary
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

`50-handoff-or-stop`'s delegation to `dispatch` is confirmed accurate and
requires no change; `dispatch`'s own STAND verdict (unchanged package
identity and stage names) means this package's delegation citation does
not itself become newly dangling as a side effect of `dispatch`'s pass.

## Alternatives considered

- **Treat the `reconcile-and-cleanup-fleet` reference as a REHOME trigger
  for this package**, on the theory that a package citing a permanently
  -foreclosed procedure as "owned" is itself unsound as published.
  Rejected: the package's own stage boundaries, judgment content, and
  outcome do not depend on `reconcile-and-cleanup-fleet` existing — only
  the prose describing what happens *after* this stage's output is wrong.
  Correcting prose in place is `convention.md` §2's normal content-edit
  path, not grounds to move or restructure the package — the same
  reasoning `dispatch`'s own pass applied to its two dangling references
  (mirrors `BU-VAS-10`'s FOLD disposition exactly).
- **Fold `60-reconcile`'s scope note entirely, on the theory that once
  `reconcile-and-cleanup-fleet` is known not to exist, the note's whole
  purpose (distinguishing this stage's narrow scope from fleet-wide
  cleanup) collapses.** Rejected: the scope note's actual function —
  clarifying that `60-reconcile` reports only this plan's repo-set facts,
  not fleet-wide state — is independent of whether a fleet-cleanup
  procedure exists to consume those facts; the note still correctly
  prevents this stage from silently absorbing fleet-wide reconciliation
  scope. Only the specific named-procedure claim needs correction, not the
  scope discipline itself.
- **Silently rewrite the dangling reference on this producer's own
  authority**, since the correct characterization is evident from the
  already-published re-homing record. Rejected per this Work's brief
  ("Produce the files and stop — you are the producer, not the reviewer")
  and per `reference/proposal-icm-r-procedure-authority.md` §8.9/§4.9: a
  producer does not self-promote its own output; the remediation is
  recorded here for the independent review/reconcile step to land.
- **Treat `20-define-dependency-order`'s cycle-rejection rule as PL-6
  deterministic machinery** rather than PL-5 judgment, since "reject
  cycles" sounds mechanical. Rejected: detecting whether a cycle reflects
  a genuinely coupled contract (versus a modeling error that should just
  be redrawn) and then designing the contract artifact or compatibility
  phase that breaks it requires the same class of judgment §6.4 describes;
  only the "never let a cycle reach dispatch" half is a fixed rule (J5),
  the "how to break a genuine one" half is not mechanical.

## Final disposition
STAND

## Validation evidence

- Source-valid: every existing behavior-unit citation in this package's
  five stage `CONTEXT.md` files was read in full and traced to its already
  -archived N1 provenance (`docs/gauntlet/promoted-provenance/
  cross-repo-work.md`); no new citation was fabricated for this pass.
- Placement-valid: every stage's already-recorded PL-5 rung
  ("actor-stage (§6.4, judgment)") was independently re-derived from the
  Placement Ladder in this pass and confirmed, not merely copied from the
  package's own table; N1 adjudication A4's six-to-five fold was
  independently re-checked against §6.3's reimplementation test rather
  than accepted on the package's self-report.
- Authority-valid: **not yet** — this is precisely what `BU-CRW-08`/`09`
  found missing. The package cannot be called authority-valid
  (`reference/proposal-icm-r-procedure-authority.md` §9.1 claim 3) until
  the three remediation items under "Surviving package design" land.
- Delegation-valid (this pass's specific check, mirroring the task the
  brief assigned `dispatch`'s pass for its own two citations): both of
  this package's cited cross-workflow references were independently
  verified against the *current* `.sergeant/workflows/` directory listing,
  `.sergeant/index.md`'s catalog, `docs/icm/re-homing-record-2026-08-12.md`,
  and `dispatch`'s own ICM-R3 verdict — `dispatch` is live, unchanged, and
  the citation to it is accurate (`BU-CRW-05`); `reconcile-and-cleanup
  -fleet` does not exist as a live package or draft and its multi-repo half
  is doctrinally foreclosed, making the "adjacent, owned procedure"
  characterization inaccurate (`BU-CRW-06`/`07`). This is the exact
  dangling-reference defect class `dispatch`'s reviewer pass asked this
  package's own pass to check.
- Structurally valid: all five stage directories, their `output/
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
