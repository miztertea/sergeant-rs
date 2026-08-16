# Package adjudication: vet-external-skill

ICM-R3 full-reconciliation pass, `docs/adr/0013-icm-r0-owner-rulings.md`
decisions 6-9; method per `reference/proposal-icm-r-procedure-authority.md`
§8; record shape per `docs/icm/record-shapes.md` §6. Producer pass only —
independent review is a separate step (§8.11 of the proposal; §6.2/6.3 of
`docs/icm/convention.md`) and has not run yet. This record and (if the
verdict required it) any revised draft content are themselves draft —
neither is self-promoting (ADR 0013 decision 6, decision 7).

Package-specific hint carried in this Work's brief: "SS12.7 hypothesis:
likely STAND. Verify directly." Verified directly against the package's
current content below, not assumed — see Behavior-unit dispositions and
Final disposition.

## Original intention

Vet an external skill through a fixed, ordered sequence before adopting
it, and keep already-adopted skills updated through the same discipline
(`.sergeant/workflows/vet-external-skill/CONTEXT.md` "Purpose"; `index.md`
description). Promoted into the N1 reference corpus as candidate **W34**
(`docs/gauntlet/contracts/N1.md`,
`docs/icm/promotion-spec-2026-08-11.md`), with a full behavior-unit
citation trail already archived at
`docs/gauntlet/promoted-provenance/vet-external-skill.md`. This ICM-R3
pass does not re-run that N1 extraction; it applies the Placement and
Bounded-Judgment ladders on top of the already-cited N1 content and checks
the package's compliance with ADR 0013's twelve rulings, following the
same method already exercised on this pilot's sibling package
`validate-and-ship` (`docs/gauntlet/runs/icm-r2/validate-and-ship/
adjudication-draft.md`), whose findings this record's own gap-finding
sections mirror where the same class of defect recurs.

The package's own `CONTEXT.md` calls it "a strong candidate for the
smallest complete reference workflow in the corpus" — five ordered
checkpoints (`00`, `10`, `20`, `30`, `50`) plus two mutually exclusive
update variants (`60-update-managed`/`60-update-owned`) reached only when
refreshing an already-adopted skill.

## Current trigger and outcome

One linear stage list (`workflow.toml`: `00-read-source`,
`10-confirm-provenance`, `20-check-actions`, `30-verify-no-conflict`,
`50-test-in-disposable-copy`, `60-update-managed`, `60-update-owned`),
seven stages, no `40-*` (folded at N1 adjudication A4; see below), no
renumbering.

- **Initial-vet entry**, at `00-read-source`: before adopting an external
  skill, walk `00` through `50` in order.
- **Update entry** (documented in package prose, not expressed in
  `workflow.toml`'s single linear list — see "The two-entry structural
  tension" below), at `60-update-managed` *or* `60-update-owned`
  depending on how the already-adopted skill is managed; the two are
  described as "mutually exclusive."

Outcome: the skill's complete instructions and scripts are read; its
source, update mechanism, and side-effect surface (filesystem, shell,
network, Git, credentials) are known; it does not conflict with
repository `AGENTS.md` or safety policy; its source is pinned/locked
where supported; it is proven in a disposable copy before broad
installation; and, for an already-adopted skill being refreshed, no
update is accepted without inspecting a diff/lock-file change
(skills.sh-managed) or without a reviewed PR and a passing test suite
(Sergeant-owned).

## Driver and admission boundary

Driver: **stage actor**, both entries. Admission boundary: **in-work** —
the workflow receives an already-admitted intent ("vet this specific
external skill" or "update this already-adopted skill") and executes
durably from that admitted intent to a terminal, meaningful-independent-
of-conversation result (an accept/reject/pin/test verdict), matching
PL-4's own test. It passes the execution-surface test
(`convention.md` §2a): "would a human type `sgt run 'vet skill X before
adopting it' --workflow vet-external-skill`?" — yes. This is also
dual-use-relevant procedure (supply-chain vetting of third-party
instructions and scripts before they run with this repository's
credentials); nothing about that changes the driver/admission answer —
it sharpens which decisions below must land at J5/J0 rather than J2.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-VES-01 | `CONTEXT.md` (Purpose) / `BU-P1-119`, `reference/sergeant-upstream/docs/skills.md` L124-131 — fixed six-step sequence must complete before an external skill is adopted | PL-4 | J5 (governing: no external skill is adopted without completing this sequence — the whole package exists to enforce this prohibition) | STAND | `vet-external-skill` (workflow) |
| BU-VES-02 | `00-read-source/CONTEXT.md` / `BU-P1-120` — read the skill's complete SKILL.md and referenced scripts, not sampled | PL-5 | J2 (delegated: what counts as "referenced scripts" — following the skill's own reference graph to completion rather than reading only the top-level file) | STAND | `00-read-source` |
| BU-VES-03 | `10-confirm-provenance/CONTEXT.md` / `BU-P1-121` — confirm source and update mechanism | PL-5 | J2 (delegated: judging whether a claimed source and update mechanism are actually confirmable from available evidence) with an unstated **J0** carve-out this stage's own content omits (see "The unconfirmable-provenance gap" below) | STAND, with the J0 gap noted for remediation | `10-confirm-provenance` |
| BU-VES-04 | `20-check-actions/CONTEXT.md` / `BU-P1-122` — check filesystem, shell, network, Git, and credential actions across five named categories | PL-5 | J2 (delegated: assessing the actual side-effect surface from source inspection) with an unstated **J0** carve-out this stage's own content omits (see "The undelegated-severity gap" below) | STAND, with the J0 gap noted for remediation | `20-check-actions` |
| BU-VES-05 | `30-verify-no-conflict/CONTEXT.md` / `BU-P1-123` — the skill does not conflict with repository `AGENTS.md` or safety policy | PL-5 | J5 (governing: no adopted skill may contradict repository instruction or safety policy — same shape as `AGENTS.md` §3 rule 1's own invariant boundary) + J2 (delegated: judging whether a given instruction or action is in fact a conflict) | STAND | `30-verify-no-conflict` |
| BU-VES-06 | `50-test-in-disposable-copy/CONTEXT.md` / `BU-P1-125` — test in a disposable repository or worktree before broad installation | PL-5 | J5 (governing: no broad installation without a prior disposable-copy test — parallel to `validate-and-ship/20-select-intent-transport`'s "never bypass a gate" shape) + J2 (delegated: judging whether the disposable-copy run is representative enough to trust) | STAND | `50-test-in-disposable-copy` |
| BU-VES-07 | `50-test-in-disposable-copy/CONTEXT.md` Helper section — pin/lock the source where the installer supports it (folded `40-pin-source`, N1 adjudication A4) | PL-6 | J5 (governing: pin wherever tooling allows — mechanical, no alternative to choose among) | STAND — folding already correctly executed; no further placement change needed | `50-test-in-disposable-copy` (helper) |
| BU-VES-08 | `60-update-managed/CONTEXT.md` / `BU-P1-126` — for skills.sh-managed skills, rerun the official installer and inspect the diff and updated lock file before accepting changes | PL-5 | J2 (delegated: accept/reject the update after inspecting its diff and lock-file change) with an unstated **J0** carve-out this stage's own content omits (see "The silent-update-acceptance gap" below) | STAND, with the J0 gap noted for remediation | `60-update-managed` |
| BU-VES-09 | `60-update-owned/CONTEXT.md` / `BU-P1-127` — for Sergeant-owned skills, update through a reviewed PR and run `tests/instruction-policy-test.sh` plus the full Sergeant test suite | PL-5 | J5 (governing: no Sergeant-owned skill change ships without a reviewed PR and a passing instruction-policy test plus full suite) | STAND at package-identity level; the cited test path is a **source-fidelity defect** in this repository — see "The unresolvable test-path citation" below | `60-update-owned` |
| BU-VES-10 | All seven stage `CONTEXT.md` files — uniform `## Judgment required` boilerplate paragraph; no stage names J2 decision classes, J1 local choices, or J0 escalation triggers in the required shape | N/A (authoring-format compliance, not a placement question) | J5 (ADR 0013 decision 4 + `docs/icm/convention.md` §6.1: every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section "always... omission is never ambiguous" — governing requirement this package predates and does not yet satisfy) | STAND (package identity correct; in-place content amendment required — see Surviving package design) | all seven stage `CONTEXT.md` files |
| BU-VES-11 | `CONTEXT.md` (L1) — no `## Authority envelope` section exists | N/A | J5 (`convention.md` §6.1: every workflow Layer-1 `CONTEXT.md` carries an `## Authority envelope` section) | STAND, in-place amendment required | `CONTEXT.md` |
| BU-VES-12 | `CONTEXT.md` line 34 and line 38 — both cite a co-located `provenance.md` that does not exist anywhere under `.sergeant/workflows/vet-external-skill/` | N/A (citation defect, not a placement question) | J5 (`record-shapes.md` §1a rule 1 / self-check discipline: a contract-bearing or navigational reference that does not resolve is a violation) | STAND at package-identity level; **FOLD** (correct the reference in place — point to `docs/gauntlet/promoted-provenance/vet-external-skill.md`, the file that actually carries this content) | `CONTEXT.md` |
| BU-VES-13 | `workflow.toml` stage list vs. package prose describing `60-update-managed`/`60-update-owned` as "mutually exclusive" "alternate entries" | N/A (structural-model question, not a placement question) | J0 for this producer — see "The two-entry structural tension" below | STAND at package-identity level; unresolved structural question recorded, not silently answered | `workflow.toml`, `CONTEXT.md`, both `60-*` stages |

## The unconfirmable-provenance gap (BU-VES-03, full record)

`10-confirm-provenance/CONTEXT.md`'s Behavior contract is a single
unconditional imperative — "Confirm the external skill's source and
update mechanism" — with no stated escalation path for the case where
provenance *cannot* be confirmed (an unsigned tarball, an anonymous
GitHub fork, a source that does not state how it is updated).
Proceeding to `20-check-actions` and beyond with an unconfirmed source is
exactly the kind of risk-changing choice the Bounded-Judgment Ladder
exists to keep out of J1/J2 silence.

**Rungs checked:** J5 — no governing constraint in this stage's own
content requires halting on unconfirmable provenance (the whole
package's existence is itself downstream evidence that adopting an
unvetted skill is against policy, but that is the workflow-level
constraint BU-VES-01 already carries, not a stage-local one this stage
restates). J4 — no explicit user/Work decision is visible to this stage
that would authorize proceeding without confirmed provenance. J3 — no
settled record inside the package states what happens when provenance is
unconfirmable. J2 — the stage's Behavior contract names "confirm the
source and update mechanism" but not "decide what to do if you cannot."
J1 — does not apply; whether to continue vetting an unconfirmable-source
skill changes the acceptance boundary of the whole procedure, not a
local, reversible implementation detail.

**Conclusion: J0**, as the stage's content stands today. This producer
does not invent the missing escalation clause's exact wording (that is
the remediation item, not this record's job); it records that the gap
exists and that the observably correct behavior — stop and ask rather
than silently proceed on an unconfirmed source — is not currently stated
anywhere the acting harness would read it.

## The undelegated-severity gap (BU-VES-04, full record)

`20-check-actions/CONTEXT.md` names five categories to check (filesystem,
shell, network, Git, credential) but, like `10-confirm-provenance`, states
no consequence for what is found — no clause distinguishing "found nothing
concerning, proceed" from "found the skill exfiltrates credentials over
the network, stop." This is the same shape of gap the proposal's own
Finding ICMR-F4 (§3.4) calls out `validate-and-ship/40-drive-gates` for
having *solved* (auto-fix/no-op vs. ask-user); this stage has not yet
solved it.

**Rungs checked:** J5 — no in-stage governing text names credential
exposure, arbitrary network egress, or destructive filesystem/Git actions
as an automatic halt condition, though `30-verify-no-conflict`
immediately downstream is where a conflict with safety policy would in
principle be caught — meaning this specific stage currently defers the
entire judgment to the next stage without saying so. J4 — no explicit
Work decision addresses this. J3 — no settled record. J2 — "check the
actions" is delegated; "decide what checked actions mean" is not named as
a decision class. J1 — does not apply for the same reason as BU-VES-03:
the finding changes whether adoption proceeds, which is not local or
reversible once broad installation has happened.

**Conclusion: J0** for the undelegated case (a finding severe enough that
continuing would be irresponsible without a stop), same procedure as
above: record, do not invent the missing clause.

## The silent-update-acceptance gap (BU-VES-08, full record)

`60-update-managed/CONTEXT.md`'s Behavior contract requires inspecting
"the diff and updated lock file before accepting changes," and its
Judgment-required prose correctly identifies the accept/reject decision
as the genuine checkpoint (this is why N1 adjudication A4 kept and
reclassified this stage rather than demoting it as machinery). What the
stage does not state is what happens when inspection finds something
that should not be silently accepted — a scope-expanding permission, an
unexpected new network call, a maintainer change. This is structurally
identical to `validate-and-ship/40-drive-gates`'s already-solved
auto-fix/no-op-vs-ask-user split (BU-VAS-08) and to the two gaps above:
the accept path is named, the stop-and-escalate path is not.

**Rungs checked:** J5 — none in-stage. J4 — none. J3 — none. J2 — accept/
reject after inspection is named; escalate-on-suspicious-diff is not.
J1 — does not apply (a supply-chain-relevant accept/reject decision is
exactly public/external-behavior-changing, not local).

**Conclusion: J0** for the case inspection finds something that should
not be silently accepted, same procedure as above.

## The unresolvable test-path citation (BU-VES-09, source-fidelity defect)

`60-update-owned/CONTEXT.md`'s Behavior contract names a specific path,
`tests/instruction-policy-test.sh`, "plus the full Sergeant test suite,"
lifted verbatim from `reference/sergeant-upstream/docs/skills.md`
L142-144. Checked directly against this repository's own current tree:
`tests/instruction-policy-test.sh` does not exist under this repository's
own `tests/` (`tests/` here contains `estate_routes.rs`, `m1_event_core.rs`
… `t2_workflow_catalog.rs` — no `instruction-policy-test.sh`); the only
file with that name anywhere in this repository lives at
`reference/sergeant-upstream/tests/instruction-policy-test.sh`, which
`docs/DEVELOPMENT.md` treats as frozen evidence of the *source* project
being reconciled, not this repository's own live tooling (the same
source/live-repo conflation `docs/gauntlet/runs/icm-r2/validate-and-ship/
review.md` independently found at BU-VAS-06, disputed verdict). If this
workflow is ever run against this repository's own owned skills, the
literal instruction as written does not resolve to a runnable check here.
This is recorded as a defect for remediation (correct the path, or state
that the instruction is generic per-repository guidance and the actual
runnable check is whatever that target repository's own equivalent test
is called), not resolved by this producer inventing a substitute path on
this repository's behalf.

## The two-entry structural tension (BU-VES-13, full record)

`CONTEXT.md`'s stage table (and the workflow's own "Notes for reviewers")
describes `60-update-managed` and `60-update-owned` as "two mutually
exclusive update variants ... reached only when refreshing an
already-adopted skill," i.e., not part of the initial `00`→`50` vetting
walk. `workflow.toml`, however, declares one single linear
`stages = [...]` list containing all seven stages in the order
`00, 10, 20, 30, 50, 60-update-managed, 60-update-owned` with no
conditional or branching grammar (`convention.md` §2a; this milestone's
hard boundary against `workflow.toml` grammar changes;
`validate-and-ship`'s own already-accepted precedent, quoted directly in
that package's `20-select-intent-transport/CONTEXT.md` line 61: "per
convention.md's single-linear-stage-list model (no engine-level branching
exists at this milestone)").

`validate-and-ship` resolves the same shape of tension by defining two
named **entry points** into one shared ordered list — a run starts at a
different index depending on which caller invoked it, but every stage
from the entry point onward is still walked in the declared order, and
no stage is ever skipped once entered. Applying that same reading here
is consistent and is this record's working assumption: an initial-vet
run starts at `00-read-source` and stops after `50-test-in-disposable-
copy` (never reaching either `60-*` stage); an update run starts directly
at whichever `60-*` stage matches how the already-adopted skill is
managed and only that one stage runs. Under that reading nothing here is
actually broken — it is the same already-precedented, already-accepted
entry-variant shape, not a new structural defect.

What this producer does **not** independently confirm is whether "starts
at a different index and stops before the declared final stage" is
something the *current engine* actually does, versus something both this
package's and `validate-and-ship`'s prose *assert* without an engine-level
mechanism behind it. `reference/proposal-icm-r-procedure-authority.md`
§3.1's own description of stage execution (a Work is bound to one pinned
workflow and walks its declared stage order) does not by itself confirm
partial/subset admission is supported, only that each declared stage gets
fresh execution when reached. This producer checked the proposal's stage-
execution description (§3.1) and did not find it either confirms or rules
out starting a Work's walk at other than the first declared stage, or
terminating before the last.

**Rungs checked:** J5 — no governing text settles whether the engine
supports non-first-stage entry or early termination; the hard boundary
against `workflow.toml` grammar changes governs the *representation*
question but not this factual one. J4 — not applicable, this is a
factual/engine-capability question, not a decision. J3 — the closest
candidate is `validate-and-ship`'s own already-published assertion of the
same pattern, but an unverified sibling package's own unreviewed
assertion is not itself a settled record (`bounded-judgment.md` J3: "A
draft, self-authored output... does not qualify"). J2 — this stage/
package does not delegate "does the engine support this" to itself; that
is a fact about the runtime, not a judgment call within the package's own
authority. J1 — does not apply.

**Conclusion: J0** — not a defect in this package specifically (it
follows an already-accepted sibling's precedent faithfully), but an
engine-capability fact this producer is not positioned to verify from
content alone and does not guess. Recorded here rather than either (a)
silently asserting the entry-variant reading is engine-verified, or (b)
silently reclassifying the package as broken on an unconfirmed premise.
Recommended next step: an execution-valid run of `vet-external-skill`
(§9.3 of the proposal — needs to be exercised as a real Work, per the
same "execution-valid: out of scope for this producer pass" note
`validate-and-ship`'s own record carries) would settle this by observing
whether an update-entry Work actually starts at a `60-*` stage and stops
there.

## Surviving package design

No stage moves, merges, splits, or renames. The seven-stage list, the
two documented entry variants, and every already-cited N1 behavior unit
remain correctly placed at PL-4 (package) / PL-5 (each stage) / PL-6 (the
one identified helper). The package requires **in-place content
amendment**, not restructuring:

1. Add a `## Bounded judgment` section (per `convention.md` §7.3 /
   `bounded-judgment.md`) to each of the seven stage `CONTEXT.md` files,
   replacing (or supplementing, per house style once a template lands)
   the current `## Judgment required` boilerplate with named J2
   delegations, J1 local choices, and J0 escalation triggers specific to
   that stage. Three of the seven (`10-confirm-provenance`,
   `20-check-actions`, `60-update-managed`) need a genuinely new J0
   clause the current prose omits entirely — see the three gap records
   above — not merely a restatement of existing content, unlike most of
   `validate-and-ship`'s equivalent amendment.
2. Add a `## Authority envelope` section to the workflow-level
   `CONTEXT.md` (per `convention.md` §7.2).
3. Correct the dangling `provenance.md` reference at `CONTEXT.md` lines
   34 and 38 to point at
   `docs/gauntlet/promoted-provenance/vet-external-skill.md`, the file
   that actually carries this package's stage-to-behavior-unit mapping.
4. Correct or generalize the `tests/instruction-policy-test.sh` path
   cited in `60-update-owned/CONTEXT.md` so it does not assert a
   specific file this repository does not itself contain.
5. Leave a citable placeholder at the two-entry structural tension
   (BU-VES-13) pending an execution-valid run; do not invent an engine-
   capability claim this producer cannot verify from content alone.

None of these five amendments changes which package owns the behavior,
so none triggers this ADR's REHOME/SPLIT/HARVEST draft-and-rehome step
(`docs/adr/0013-icm-r0-owner-rulings.md` decision 6; task brief). They
are recorded here as the concrete remediation this adjudication found,
for the owner/reviewer to schedule — matching exactly how
`validate-and-ship`'s own ICM-R2 pass handled the same class of gap.

## Inputs and outputs

Inputs: as declared in each stage's own Inputs table — all seven comply
with `record-shapes.md` §1a (verified during Inventory: each stage names
exactly the prior stage's `output/README.md`, or, for the two `60-*`
alternate entries, explicitly declares "no contract-bearing upstream
dependency beyond this workflow's ordering," which is accurate given
they are alternate entries rather than continuations of `50`'s output).
No contract-bearing dependency was found undeclared.

Outputs: `output/README.md` in each stage declares its expected artifact
and disposition. Four of seven are `evidence` (Work-branch record only:
`00-read-source`, `10-confirm-provenance`, `20-check-actions`,
`30-verify-no-conflict`); three are `promote` (workflow deliverable:
`50-test-in-disposable-copy`, `60-update-managed`, `60-update-owned`) —
consistent with there being three distinct terminal stages across the
two entry variants (an initial vet terminates at `50`; an update
terminates at whichever `60-*` it entered). No violation found in the
Layer 4 declarations. As with `validate-and-ship`, no deterministic
finalize step is named for any of the three `promote` stages
(`docs/icm/promotion-spec-2026-08-11.md` §1 finalize gap, `convention.md`
§1a open question 1) — not a blocker on the convention's own current
text, recorded per the spec's curation rule.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (not a draft) — its structural and provenance
identity does not change. The five remediation items above are ordinary
content edits to an admitted workflow and should go through this
repository's normal review path for workflow content changes, not a new
draft-and-promote cycle, per `docs/icm/convention.md` §2 (the
draft/admitted split governs *new or substantially rewritten* content;
adding required sections and correcting citations to an already-admitted
stage's `CONTEXT.md` is neither). Per ADR 0013 decision 6, only the
promotable form of this change (once actually made) needs independent
review before it lands — this adjudication record itself, being ICM-R3
producer evidence, needs the reconciliation's own reviewer step
(`reference/proposal-icm-r-procedure-authority.md` §8.11) before its
findings are treated as settled.

## Alternatives considered

- **REHOME the whole package to a Captain skill**, on the theory that
  vetting an external skill is inherently a live, conversational,
  judgment-heavy activity closer to how a human reviews a dependency.
  Rejected: the package passes the execution-surface test
  (`convention.md` §2a) cleanly — each stage receives a bounded intent
  (which skill, from where), produces a durable evidence artifact, and
  the whole sequence terminates in a meaningful, conversation-independent
  result (adopt / reject / accept-update / reject-update). Nothing about
  it requires live dialogue as its primary product; it can and should run
  as a dispatched Work, the same reasoning `validate-and-ship`'s own
  Alternatives section already applied and N1 adjudication A5 already
  litigated for a sibling package.
- **HARVEST the credential/network/destructive-action checks
  (`20-check-actions`) into `security-review`'s own package**, on the
  theory that action-surface checking is a general security-review
  technique, not something specific to skill vetting. Rejected without a
  fuller read of `security-review` than this producer's brief scoped:
  the behavior here is bounded specifically to "an external skill about
  to be adopted," has its own upstream citation (`BU-P1-122`) independent
  of any security-review package, and demoting it to a shared technique
  would lose the fixed-sequence guarantee (§Purpose: "vetted through a
  fixed sequence") that is this package's entire reason for existing as
  one ordered whole rather than a menu of checks. If a future pass
  reconciling `security-review` finds a genuinely reusable technique here,
  that is a PL-3 (actor skill / shared method) candidate to raise then,
  not a reason to dissolve this package's own ordering now.
- **Treat the three J0 gaps (BU-VES-03/04/08) as engine-gap (PL-7)
  claims.** Rejected: nothing about any of the three requires the
  runtime to own a new durable fact — each requires this package's own
  stage content to state an escalation clause it currently omits. Lower
  rungs (a stage-local J0 clause, exactly `40-drive-gates`'s own already-
  proven pattern) have not been attempted yet, so PL-7 is unreached per
  the ladder's own first-honest-rung rule (proposal §4.8).
- **Silently draft the three missing J0 clauses' exact wording on this
  producer's own authority**, resolving the gaps rather than recording
  them. Rejected: per `bounded-judgment.md`'s own J0 procedure, a
  producer at J0 states the gap and may offer a recommendation as
  evidence, but the specific escalation wording is exactly the kind of
  authoring judgment that belongs with whoever lands the amendment
  (likely alongside the `## Bounded judgment` section template ICM-R1
  established), not invented here as a fait accompli inside an
  adjudication record.
- **Treat the two-entry structural tension (BU-VES-13) as settled by
  `validate-and-ship`'s precedent, no further note needed.** Rejected:
  precedent from an unreviewed sibling producer pass is not itself a J3
  settled record (`bounded-judgment.md` J3 explicitly excludes
  self-authored, unreviewed output); the honest position is "this is the
  working assumption, consistent with precedent, not independently
  engine-verified," which is what this record states.

## Final disposition
STAND

## Validation evidence

- Source-valid: every existing behavior-unit citation in this package's
  seven stage `CONTEXT.md` files was read in full and traced to its
  already-archived N1 provenance
  (`docs/gauntlet/promoted-provenance/vet-external-skill.md`); the
  archived provenance file's own citations were independently spot-
  checked against `reference/sergeant-upstream/docs/skills.md` L100-153,
  confirming byte-for-byte agreement with the "Before adopting an
  external skill" and "Updating skills" sections; no new citation was
  fabricated for this pass.
- Placement-valid: every stage's already-recorded PL-5 rung
  (`actor-stage (§6.4, judgment)`) was independently re-derived from the
  Placement Ladder in this pass and confirmed, not merely copied from
  the package's own table; the one PL-6 helper (pin/lock) was likewise
  re-derived and confirmed.
- Authority-valid: **not yet** — this is precisely what BU-VES-03/04/08
  (missing J0 clauses) and BU-VES-10/11 (missing required sections)
  found lacking. The package cannot be called authority-valid
  (`reference/proposal-icm-r-procedure-authority.md` §9.1 claim 3) until
  the five remediation items under "Surviving package design" land.
- Structurally valid: all seven stage directories, their `output/
  README.md` declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly. Two citation
  defects were found (BU-VES-09's unresolved test path, BU-VES-12's
  dangling `provenance.md` reference) — both recorded, neither silently
  fixed by this producer in the live package (STAND with in-place
  amendment recorded, not executed, matching this ADR's producer/
  reviewer separation).
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the
  package; `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims (needs_input on a real/scripted J0 case,
  operation without Captain present, and — specific to this package —
  whether an update-entry Work actually starts at a `60-*` stage and
  stops there per BU-VES-13) remain to be measured separately.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7); it
  does not self-promote.
