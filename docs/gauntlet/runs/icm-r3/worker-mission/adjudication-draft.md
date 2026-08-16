# Package adjudication: worker-mission

ICM-R3 full-reconciliation pass, package `worker-mission`
(`docs/adr/0013-icm-r0-owner-rulings.md`; method
`reference/proposal-icm-r-procedure-authority.md` §8 — Contract, Inventory,
Harvest, Normalize, Placement classification, Authority classification,
Synthesis; §10.4 ICM-R3 scope, final wave: `worker-mission`'s `20-implement`
delegates conditionally to `diagnose-bug`, `prototype`, `tdd`, `implement`,
or `deepen-module` — all five already reconciled this pass (four settled
STAND, one — `tdd` — disputed REHOME) — so this pass runs against a mostly-
settled but not-fully-settled delegate graph, not blind). Record shape:
`docs/icm/record-shapes.md` §6. Producer draft only — independent
adversarial review (§8.11 of the proposal) and Captain's
reconcile-and-publish pass (§8.12) have not run. This record is itself
draft; it does not self-promote (`docs/adr/0013` decision 6).

This pass read all five delegate packages' ICM-R3 records in full before
forming its own view:

- `docs/gauntlet/runs/icm-r3/diagnose-bug/{adjudication-draft.md,review.md}`
  — **STAND, confirmed** (one additional FOLD item: dangling
  `provenance.md` reference).
- `docs/gauntlet/runs/icm-r3/prototype/{adjudication-draft.md,review.md}`
  — **STAND, confirmed** (one sub-disposition correction, does not change
  the package verdict).
- `docs/gauntlet/runs/icm-r3/tdd/{adjudication-draft.md,review.md}` —
  producer: **REHOME** (PL-3, folds into `.sergeant/common/contexts/
  {tdd,test-quality}.md`); independent reviewer: **DISPUTED** — not a
  reversal to STAND, but a finding that the producer's "Alternatives
  considered" never weighed the PL-7 engine-gap alternative that
  `record-shapes.md` §5's own canonical worked example describes almost
  verbatim for `tdd`'s exact shape (two genuinely independent direct
  parents — `implement` and `worker-mission` — each needing their own
  fresh-execution seam/red-green checkpoints). **Unresolved as of this
  pass.**
- `docs/gauntlet/runs/icm-r3/deepen-module/{adjudication-draft.md,
  review.md}` — **STAND, confirmed** (two J-rung citation corrections,
  does not change the package verdict).
- `docs/gauntlet/runs/icm-r3/implement/{adjudication-draft.md,review.md}`
  — producer: **STAND** with FOLD-grain internal restructuring
  (per-stage Bounded-judgment sections, a workflow-level Authority
  envelope, a corrected provenance pointer, a corrected `code-review`
  delegation mechanism, and a filed engine-gap claim,
  `draft/engine-gap-nested-workflow-invocation.md`); independent reviewer:
  **CONFIRMED (STAND)**, with two narrow corrections that do not change
  the verdict. `implement` itself delegates to `tdd` (from
  `10-implement-with-tdd`) and to the settled `code-review` (from
  `30-review`), and therefore already faced the identical problem this
  pass faces: how should a package that cites `tdd` treat a citation whose
  disposition is disputed, not settled? `implement`'s answer — reasoned
  independently below rather than copied — is the direct precedent this
  pass follows for its own `tdd` citation.

## Original intention

`.sergeant/workflows/worker-mission/CONTEXT.md`: "From a rendered brief,
produce a merged-ready change with evidence — the contract a dispatched
worker delivers against." Candidate **W9** from the N1 reference-corpus
decomposition (`docs/gauntlet/contracts/N1.md`), decomposed from
`reference/sergeant-upstream` per `reference-corpus/synthesis.md` §1,
promoted per `docs/icm/promotion-spec-2026-08-11.md` with a full
behavior-unit citation trail archived at
`docs/gauntlet/promoted-provenance/worker-mission.md`. The package is the
worker-side contract: everything a dispatched worker must do, from
pinning scope through publishing its result, once a brief exists — it does
not decide whether work should be dispatched (that is `dispatch`'s and,
upstream of `dispatch`, `task-intake-and-route`'s job).

## Current trigger and outcome

Trigger (`CONTEXT.md`, `index.md`, `workflow.toml`): "A worker starts
against a rendered brief." Outcome as packaged: four ordinary actor stages
(`workflow.toml`: `10-triage-and-route`, `20-implement`,
`30-independent-review`, `40-escalate-or-continue`) — scope pinned and the
work triaged into one of five disciplines (folded `00-pin-scope` helper
ahead of `10-triage-and-route`'s own judgment, N1 adjudication A4); the
chosen discipline runs to completion (`20-implement`); every axis named in
the brief's authoritative list reviews the result as separate,
non-contaminating parallel passes (`30-independent-review`); then either a
new monotonic-generation gate is published and the handshake completed, or
the mission concludes and handoff evidence is recorded (folded
`50-publish-result` helper after `40-escalate-or-continue`'s own judgment,
N1 adjudication A4). `40-escalate-or-continue`'s output carries `promote`
disposition; it is the workflow's last stage.

As packaged, `worker-mission` is `status: published` under
`.sergeant/workflows/` (`index.md`), listed in the root catalog
(`.sergeant/index.md` line 46). It is dispatched by `dispatch`
(`.sergeant/workflows/dispatch/CONTEXT.md`, `80-monitor/CONTEXT.md`: brief
content "is the input to `worker-mission`") and is itself named by
`code-review/CONTEXT.md` line 25 ("invoked directly or delegated from
`worker-mission`/`implement`") as one of two packages that can delegate
into code review — though, checked directly, `worker-mission` reaches
`code-review` only indirectly, via selecting `implement` as one of its
five disciplines at `10-triage-and-route`, never as a direct delegate of
its own. `recover-stalled-worker`'s own ICM-R3 pass
(`docs/gauntlet/runs/icm-r3/recover-stalled-worker/adjudication-draft.md`)
independently checked and confirmed that its own isolation from
`dispatch` and `worker-mission` is correct — thematic worker-lifecycle
adjacency is not a shared behavioral contract — so no coherence problem
was found from that direction either.

## Driver and admission boundary

Driver: stage-actor throughout — four ordinary actor stages, no Captain
dialogue and no deterministic/execute stage
(`.sergeant/workflows/worker-mission/workflow.toml`). Admission boundary:
in-work — given an already-rendered brief (an accepted upstream artifact
from `dispatch`), `worker-mission` executes durably from admission
(triage) through a terminal, merge-relevant result (a reviewed,
handshake-completed or handoff-recorded change), independent of whether
Captain remains present. Applying §5.6's PL-4 question directly — "given
an already-defined intent, repositories, constraints, and expected
outcome, can Sergeant execute this procedure durably from admission to a
terminal result whether or not the Captain remains present?" — the answer
is yes. Nothing in this pass, or in any of the five delegate records read
above, challenges `worker-mission`'s own PL-4 rung. **Package identity and
driver STAND.**

### The `tdd` citation: independent reasoning, not inherited conclusion

`worker-mission/20-implement/CONTEXT.md`'s `## Delegation` section names
`tdd` as one of five disciplines `10-triage-and-route` may select,
identically worded (near-verbatim, confirmed by direct read) to
`implement/10-implement-with-tdd/CONTEXT.md`'s own delegation to `tdd`:

> "This stage's outcome is produced by running **\<delegate\>** to its own
> completion (context composition today — see `docs/icm/convention.md` §4
> on `@@name` versus true nested-workflow invocation, which does not exist
> yet)."

`implement`'s own ICM-R3 pass faced this exact problem — a package citing
`tdd`, whose disposition is disputed rather than settled — and reasoned to
a specific answer: correct what is dispute-independent now (the hidden
`BU-TDD-02` seam-confirmation J0 the loading stage must honor regardless
of which way `tdd`'s placement resolves), leave the delegation's
description of `tdd` itself only partially revised (still naming `tdd`,
still calling the current mechanism "context composition today"), and
explicitly flag the stage for a follow-up revision once `tdd`'s dispute
resolves — rather than adopting either disputed side.

Applying that same method to `worker-mission`'s own citation, independently
re-derived rather than copied:

1. **Is `worker-mission`'s relationship to `tdd` the same kind of citation
   `implement` had?** Checked directly: yes, in the relevant respect.
   Both are stage-level delegations using the identical undeclared
   context-composition mechanism, both predate `tdd`'s ICM-R3 dispute, and
   `tdd`'s own reviewer explicitly treats `worker-mission` as one of
   `tdd`'s "two genuinely independent direct parents" (`tdd/
   adjudication-draft.md` "Driver and admission boundary" point 2,
   confirmed again from the `implement` side at `implement/
   adjudication-draft.md` `BU-IMPL-09`). `worker-mission` is not a
   downstream consumer of `implement`'s citation — it is `tdd`'s *other*,
   independent, direct citation, not a second-hand one. This makes the
   precedent apply with at least equal force, not by analogy through
   `implement`.
2. **Does anything about `worker-mission`'s own context change the
   calculus `implement` used?** Checked for a genuine difference rather
   than assumed none: `worker-mission`'s `20-implement` is a five-way
   branch, not a single named delegation the way `10-implement-with-tdd`
   is. Only the `tdd` branch is disputed; the other four branches
   (`diagnose-bug`, `prototype`, `implement`, `deepen-module`) are each
   independently confirmed STAND by their own ICM-R3 records read above.
   This does not weaken the case for leaving the `tdd` branch alone
   pending its own resolution — if anything it sharpens the boundary: the
   four settled branches can be corrected in full now (their delegates'
   placement is not in question, mirroring `implement`'s treatment of its
   settled `code-review` citation), while only the `tdd` branch needs the
   narrower, dispute-independent-only correction. A single Delegation
   section cannot receive two different levels of revision without being
   split into named sub-clauses — which this pass does (see "Surviving
   package design").
3. **Would adopting `tdd`'s disputed REHOME fold notes now be the right
   move, since a draft fold note already exists for this exact file**
   (`docs/gauntlet/runs/icm-r3/tdd/draft/fold-notes/
   worker-mission-20-implement.md`)? Read that fold note directly: it is
   explicitly conditioned on REHOME being accepted ("Not a live edit... the
   `worker-mission` package is not itself part of this ICM-R3 pass's
   assigned scope"; its proposed replacement text presupposes `@@tdd` and
   `@@test-quality` exist as admitted shared contexts). They do not yet —
   `.sergeant/common/contexts/tdd.md` is itself unpublished, disputed draft
   content (`tdd/review.md`'s "Do not promote REHOME as drafted"). Adopting
   that fold note's replacement text now would strand `worker-mission`'s
   `20-implement` stage on a reference (`@@tdd`) that may never be admitted
   in that form, exactly the risk `implement`'s own pass identified and
   rejected for its own `10-implement-with-tdd` stage. **Rejected for the
   same reason, independently re-confirmed against `worker-mission`'s own
   content: the fold note's premise is not yet true.**

Conclusion: `worker-mission`'s own driver and PL-4 rung STAND, matching
`implement`. Its `20-implement` `## Delegation` section requires revision,
but — following the same reasoning `implement` used, independently applied
here — that revision is uneven: the four settled branches can be corrected
in full; the `tdd` branch can only be corrected on the dispute-independent
point (the confirmed hidden J0 dependency, `BU-TDD-02`/`BU-TDD-04`) without
adopting either disputed disposition as fact. See "Surviving package
design" below.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| `BU-WM-01` | `CONTEXT.md`/`index.md` workflow-level trigger, purpose (workflow-level citation, `docs/gauntlet/promoted-provenance/worker-mission.md`) | PL-4 (§5.6 — admission-to-terminal-result procedure; not disputed, see "Driver and admission boundary") | J3 — settled by the rendered brief `dispatch` produces (an accepted upstream artifact); `worker-mission` does not re-decide whether the brief warrants dispatch, only executes against it | STAND | unchanged (also now stated as an `## Authority envelope` item, see `BU-WM-10`) |
| `BU-WM-02` | `10-triage-and-route/CONTEXT.md` helper: pin scope, folded from `00-pin-scope` (`BU-P7-005`, N1 adjudication A4) | PL-6 (deterministic helper) | J1 — mechanical: fetch refs, pin base commit, record base SHA/commit list/diff scope | STAND | unchanged, already correctly folded per `docs/gauntlet/promoted-provenance/worker-mission.md` "Adjudication A4" |
| `BU-WM-03` | `10-triage-and-route/CONTEXT.md` `## Behavior contract` — triage/route into one of five categories (`BU-P7-007`) | PL-5 (§5.7 — distinct artifact downstream stages depend on, independent retry unit, authority envelope changes materially at this boundary versus the stage before it) | J2 — classification among five named categories is explicitly delegated to this stage; **J0 gap found** (see finding below) | FOLD (add `## Bounded judgment` section, including the J0 clause below) | future `## Bounded judgment` in live `CONTEXT.md` |
| `BU-WM-04a` | `20-implement/CONTEXT.md` `## Delegation` — the `diagnose-bug`, `prototype`, `implement`, `deepen-module` branches | PL-4, inherited from each delegate's own settled rung (all four independently confirmed STAND this pass, see records read above) | J5 — the invocation mechanism must not misdescribe a checkpointed, authority-enveloped workflow as context composition (`docs/icm/convention.md` §4 rule 1); same defect class as `implement`'s settled `code-review` correction (`BU-IMPL-04`/`BU-IMPL-09`), independently re-derived here against these four delegates' own current structure | FOLD (revise prose in full — these four delegates' placement is not in question) | corrected `## Delegates to` clause |
| `BU-WM-04b` | `20-implement/CONTEXT.md` `## Delegation` — the `tdd` branch | Rung inherited from `tdd`'s own disputed placement — **not resolved by this pass** (out of scope; `tdd`'s dispute is a separate, still-open adjudication) | J0 the loading stage must honor regardless of `tdd`'s eventual rung: `BU-TDD-02`/`BU-TDD-04` (CONFIRMED by `tdd`'s independent reviewer, cross-confirmed by `implement`'s independent reviewer) — no test is written at an unconfirmed seam, and this stage's own contract currently never states that, only "context composition" prose that requires reading `tdd`'s full source to discover it | FOLD (revise prose only on the dispute-independent point; do not adopt the disputed REHOME fold note's `@@tdd`/`@@test-quality` replacement text) | corrected `20-implement/CONTEXT.md` sub-clause for the `tdd` branch, plus a tracking note pointing at `docs/gauntlet/runs/icm-r3/tdd/` by path |
| `BU-WM-05` | `30-independent-review/CONTEXT.md` `## Behavior contract` — brief-authoritative-axis parallel review, non-contaminating, unblended (`BU-P7-013`) | PL-5 (distinct artifact, independent retry unit; authority envelope differs materially from `20-implement` — this stage may not narrow review coverage to whatever a loaded review skill names) | J5 — coverage is fixed by the brief's authoritative axis list regardless of what a loaded review skill would otherwise cover; this is a governing constraint on the stage, not a J2 choice it may narrow | STAND | unchanged (also now stated as an `## Authority envelope` "may not decide" item) |
| `BU-WM-06` | `40-escalate-or-continue/CONTEXT.md` `## Behavior contract` — ack/accept/act-once handshake, same supervisor-scoped token per step (`BU-P7-009`) | PL-5 | J5 — exactly-once, duplicate-safe handshake is a governing correctness constraint, not a local implementation choice | STAND | unchanged |
| `BU-WM-07` | `40-escalate-or-continue/CONTEXT.md` `## Behavior contract` — monotonic per-worktree gate-generation counter persisted before the waiting status is written (`BU-P7-012`) | PL-5 (stays with the stage; the persist-before-write ordering is itself the durable checkpoint, not subordinate machinery) | J5 — ordering requirement is non-negotiable; a repeated blocker is only a new gate if the generation actually advanced | STAND | unchanged |
| `BU-WM-08` | `40-escalate-or-continue/CONTEXT.md` helper: publish result, folded from `50-publish-result` (`BU-P7-066`, `BU-P7-110`, N1 adjudication A4) | PL-6 (deterministic helper) | J5 for worktree-source verification (handoff evidence must resolve from the verified worktree, never the supervisor's ambient cwd — non-negotiable per the cited test); J1 for the bounded-wait/reporting mechanics | STAND | unchanged, already correctly folded |
| `BU-WM-09` | All four stage `CONTEXT.md` files use `## Judgment required` boilerplate, not the `## Bounded judgment` heading with named J2/J1/J0 subsections `docs/icm/convention.md` §6.1 / `docs/adr/0013` decision 4 require of every actor stage | N/A (authoring-format compliance) | J5 (governing requirement this package predates) | FOLD (replace, all four stages) | live `CONTEXT.md` files |
| `BU-WM-10` | Workflow `CONTEXT.md` has no `## Authority envelope` section | N/A | J5 — `docs/icm/convention.md` §6.1 requires one on every workflow Layer-1 `CONTEXT.md` | FOLD (add) | live `CONTEXT.md` |
| `BU-WM-11` | `CONTEXT.md` "## Provenance" says "See `provenance.md`" — no such file exists under `.sergeant/workflows/worker-mission/`; the real file is `docs/gauntlet/promoted-provenance/worker-mission.md` (confirmed: `ls .sergeant/workflows/worker-mission/` lists only the four stage directories, `CONTEXT.md`, `index.md`, `workflow.toml`) | N/A (structural — broken self-reference) | n/a | FOLD (correct pointer) | live `CONTEXT.md`, same defect class as `diagnose-bug`'s BU-DB-12 and `implement`'s BU-IMPL-08 |
| `BU-WM-12` | Structural, cross-package: `20-implement`'s delegation prose exhibits the identical "context composition today... does not exist yet" hedge already covered by `implement`'s filed engine-gap claim, generalized here across five delegation targets instead of `implement`'s one/two — and independently self-flagged inside the package's own text (`10-triage-and-route/CONTEXT.md` "Additional note": "raises engine-gap **G6** (child-procedure invocation with its own checkpoints)... representable today only by inlining the chosen discipline's stages, losing independent parent/child checkpoint and recovery visibility") | PL-7 (already claimed) | n/a | not a new claim — additional source evidence for the existing filed claim | recommend Captain's reconcile-and-publish pass add `worker-mission/20-implement` (all five branches) and its own self-flagged `G6` citation (`reference-corpus/synthesis.md` §5) to `docs/gauntlet/runs/icm-r3/implement/draft/engine-gap-nested-workflow-invocation.md`'s `source_evidence`; this pass does not edit that file (out of its own assigned package scope, same restraint `tdd`'s producer applied to `test-quality.md`'s other four consumers) |
| `BU-WM-13` | Systemic, not package-specific: `CONTEXT.md` L1 reads "Draft workflow package" while `index.md` states `status: published` | N/A | n/a | not actionable at this pass | recorded per `deepen-module`'s reviewer precedent (`docs/gauntlet/runs/icm-r3/deepen-module/review.md`): `grep -l "^Draft workflow package" .sergeant/workflows/*/CONTEXT.md` returns the same 19-package boilerplate-drift set that finding already scoped as systemic, not `worker-mission`-specific; recording it here so it is not silently lost, no disposition change |

## Finding: `10-triage-and-route`'s missing J0 for straddling candidates

`deepen-module`'s independent reviewer found a parallel gap in
`00-classify-dependencies`: a candidate whose evidence straddles more than
one of that stage's mutually exclusive categories is not cleanly resolved
by the stage's J2 classification delegation, and the correct rung is J0,
not a silently invented tie-break. `10-triage-and-route`'s own
`## Behavior contract` (`BU-WM-03`) has the identical structural shape: one
work item classified into exactly one of five mutually exclusive
categories (huge/foggy, hard bug or perf regression, uncertain design/UI,
approved feature/fix, merge/rebase conflict), each loading a materially
different procedure with a different authority envelope. Rungs checked,
same reasoning as `deepen-module`'s finding: **J5** no constraint requires
or forbids treating a straddling work item as any one category; **J4** no
user/Work decision addresses it (the brief names the work, not its
category); **J3** no settled record addresses it; **J2** the stage
delegates classification among the five categories, but classifying a
case where the work genuinely straddles two of them (e.g., a hard bug that
also requires an uncertain design call before it can be fixed) is not the
same delegation as choosing among five categories that are actually
mutually exclusive for the case at hand; **J1** does not apply, the choice
is downstream-binding (it selects which of five different procedures with
different authority envelopes runs next). **Conclusion: J0.** When
`10-triage-and-route/CONTEXT.md`'s `## Bounded judgment` section is
drafted (`BU-WM-03`'s remediation), "the work straddles more than one of
the five categories and no single classification is clearly dominant"
should be named as a `needs_input` trigger, parallel to
`deepen-module`'s own required addition for its structurally identical
five-vs-four-category branching point.

## Surviving package design

Four actor stages, unchanged in count, order, and identity —
`10-triage-and-route`, `20-implement`, `30-independent-review`,
`40-escalate-or-continue`, all `driver: stage-actor`, `worker-mission`
remains PL-4/STAND. All four gain a proper `## Bounded judgment` section
(`BU-WM-09`) and the workflow `CONTEXT.md` gains `## Authority envelope`
(`BU-WM-10`) and a corrected provenance pointer (`BU-WM-11`), mirroring the
same internal-fidelity fixes `code-review` (ICM-R2) and `implement`
(ICM-R3) already applied to themselves.

1. **`10-triage-and-route`**: gains a `## Bounded judgment` section naming
   the five-way classification as its named J2 delegation and the
   straddling-candidate case (above) as its required J0 clause. The
   folded pin-scope helper's content is unchanged.
2. **`20-implement`**: the `## Delegation` section is renamed
   `## Delegates to` and split into two sub-clauses reflecting the uneven
   revision this pass's independent reasoning requires (see "Driver and
   admission boundary" above):
   - The `diagnose-bug` / `prototype` / `implement` / `deepen-module`
     branches are corrected in full: stated plainly as separately-admitted
     PL-4 workflows, that today's best-available invocation mechanism is
     dispatching the chosen one as its own Work (not context pull-in,
     per `proposal-next-iteration-icm-workflows.md` §7.7, matching
     `implement`'s own `code-review` correction), and that true
     nested-workflow invocation (the already-filed engine-gap claim,
     `BU-WM-12`) is the eventual correct fix.
   - The `tdd` branch keeps its current wording (still names `tdd`, still
     says "context composition today... does not exist yet") but gains an
     explicit statement of the seam-confirmation J0 (`BU-WM-04b`) the
     loading stage must honor today, independent of `tdd`'s eventual
     placement, plus a tracking note pointing at
     `docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md` and
     `review.md` by path. **This branch should be revised again once
     `tdd`'s own ICM-R3 dispute resolves** — this record is deliberately
     not final on that point, matching the scope boundary `implement`'s
     own pass drew for the identical citation.
   - A new `## Bounded judgment` section is added, naming which discipline
     was selected as a J3 settled decision consumed from
     `10-triage-and-route`'s own output, not re-litigated here.
3. **`30-independent-review`** and **`40-escalate-or-continue`**: gain
   `## Bounded judgment` sections stating their respective J5 governing
   constraints (brief-authoritative coverage; exactly-once handshake;
   monotonic gate-generation ordering) as named, not left as unstated
   "Judgment required" prose. No content-level defect found in either
   stage's `## Behavior contract`.

Nothing in `worker-mission`'s own package is REHOMEd, SPLIT, HARVESTed, or
RETIREd. The two demoted-and-folded helpers (`BU-WM-02`, `BU-WM-08`) remain
exactly as N1 adjudication A4 left them; this pass found no defect in
either.

No `draft/` directory is written for this record. `worker-mission`'s
disposition is STAND, and the required in-place amendments (four
`## Bounded judgment` sections, one `## Authority envelope`, one corrected
provenance pointer, one split-and-partially-revised `## Delegation`
section) are recorded here as the remediation list for Captain's
reconcile-and-publish pass to apply directly to the live files, the same
scope this record's own deliverable spec calls for at STAND (contrast
`implement`, which also landed STAND but produced a `draft/` directory —
that reflects `implement`'s own dispatch instruction, not a difference in
this record's underlying findings).

## Inputs and outputs

Inputs consumed by this adjudication: every file under
`.sergeant/workflows/worker-mission/` (`CONTEXT.md`, `index.md`,
`workflow.toml`, all four stage `CONTEXT.md`/`output/README.md` pairs);
`docs/gauntlet/promoted-provenance/worker-mission.md`;
`reference-corpus/synthesis.md` §1 and §5 (engine-gap **G6**);
all five delegate packages' ICM-R3 records in full
(`docs/gauntlet/runs/icm-r3/{diagnose-bug,prototype,tdd,implement,
deepen-module}/{adjudication-draft.md,review.md}`, plus `implement`'s
`draft/` content and `tdd`'s `draft/fold-notes/
worker-mission-20-implement.md`); `docs/gauntlet/runs/icm-r3/
recover-stalled-worker/adjudication-draft.md` (grepped for
`worker-mission` to confirm no relationships-section gap from that
direction); `.sergeant/workflows/dispatch/CONTEXT.md` and
`80-monitor/CONTEXT.md`; `.sergeant/workflows/code-review/CONTEXT.md`
(grepped for `worker-mission`); `.sergeant/index.md` and `AGENTS.md`
(grepped for `worker-mission`); `reference/proposal-icm-r-procedure-
authority.md` §5, §6, §8, §10.4; `.sergeant/common/contexts/
bounded-judgment.md`; `docs/icm/convention.md` §§1, 1a, 2a, 4, 5, 6;
`docs/icm/record-shapes.md` §6; `docs/adr/0013-icm-r0-owner-rulings.md`.

Output: this record only — no `draft/` content, per the "Surviving
package design" scope note above.

## Review and promotion policy

Artifact class: package-adjudication record. Draft location:
`docs/gauntlet/runs/icm-r3/worker-mission/` (this record). Independent
reviewer: this ICM-R3 wave's own review step (a fresh execution, explicit
inputs, review-only contract, no edit authority over this record, per
`docs/adr/0013` decision 7). Acceptance criteria: the independently
re-derived treatment of the `tdd` citation (correct on the
dispute-independent point only, tracking `tdd`'s own unresolved dispute
rather than adopting either side) holds up against adversarial challenge,
in particular whether `worker-mission`'s status as one of `tdd`'s two
genuinely independent direct parents (not a second-hand citation through
`implement`) makes any part of `implement`'s precedent inapplicable; the
straddling-candidate J0 finding for `10-triage-and-route` (parallel to
`deepen-module`'s) holds up; no citation is fabricated. Promotion action:
Captain's reconcile-and-publish pass (§8.12) accepts/rejects/merges/parks
each finding and applies the in-place amendments directly (no `draft/`
staging needed for a STAND-only content revision); if the `tdd` branch of
`20-implement`'s delegation is revised again once `tdd`'s dispute resolves,
that follow-up is a separate, later change, not blocked on this record.
Failure/remediation: if review finds that `worker-mission`'s direct-parent
status should have produced a *different* treatment than `implement`'s
(e.g., because `worker-mission`, unlike `implement`, has no settled
sibling delegation forcing a like-for-like comparison), that specific
finding is addressed on its own; it does not disturb the four settled
branches' corrections (`BU-WM-04a`) or the authoring-format findings
(`BU-WM-09` through `BU-WM-11`), which do not depend on how the `tdd`
question resolves.

## Alternatives considered

- **Wait for `tdd`'s dispute to resolve before adjudicating
  `worker-mission` at all.** Rejected, for the same reason `implement`'s
  own pass rejected it: this pass's dispatch instruction requires reading
  the `tdd` producer draft and reviewer dispute and forming an independent
  judgment now about what `worker-mission` itself should do, not deferring
  the whole package. Four of five delegation branches, all of
  `10-triage-and-route`'s classification finding, and all of the
  authoring-format findings are answerable without waiting, since they
  rest on settled state, not on how `tdd`'s dispute resolves.
- **Adopt `tdd`'s own fold note for this exact file verbatim**
  (`docs/gauntlet/runs/icm-r3/tdd/draft/fold-notes/
  worker-mission-20-implement.md`). Rejected: that note is explicitly
  conditioned on REHOME being accepted, and the independent reviewer
  disputed REHOME rather than confirming it. Promoting content that
  presupposes `@@tdd`/`@@test-quality` exist, when
  `.sergeant/common/contexts/tdd.md` is itself unpublished disputed draft
  content, would strand `20-implement`'s `tdd` branch on a reference that
  may never be admitted in that form — the identical risk `implement`'s
  pass identified for its own citation, independently re-confirmed here
  rather than assumed to transfer automatically.
- **Treat `worker-mission`'s citation as strictly subordinate to
  `implement`'s (since `implement` is itself one of `worker-mission`'s
  five disciplines) and simply mirror whatever `implement`'s pass
  concluded, without separate reasoning.** Rejected: `tdd`'s own reviewer
  names `implement` and `worker-mission` as `tdd`'s two *independent*
  direct parents, not a citation chain — `worker-mission` reaches `tdd`
  directly from its own `20-implement` stage, not by way of `implement`'s
  `10-implement-with-tdd`. Treating the two citations as one and the same
  would understate how many places currently carry the same unresolved
  hedge and would not independently verify that `implement`'s reasoning
  actually transfers (which this pass did check, in "The `tdd` citation:
  independent reasoning, not inherited conclusion," and found it does
  transfer, but as a checked conclusion, not an assumed one).
- **REHOME or SPLIT `worker-mission` itself** (e.g., treating its mostly
  four-stage-delegated-plus-review-and-handshake content as evidence it
  should not be its own workflow). Rejected: `worker-mission`'s own PL-4
  admission-to-terminal-result test holds cleanly (see "Driver and
  admission boundary") and is not contested by any of the five delegate
  records this pass read. A workflow being thin in its middle stage
  (`20-implement` is almost entirely delegated) is not itself a placement
  defect — `implement`'s own pass made the identical argument for itself
  and it applies here with equal force.

## Final disposition
STAND

Package identity, driver, and PL-4 rung are correct and not in dispute —
see the Behavior-unit dispositions table for the individual FOLD grain
within this STAND. Internal restructuring is required before promotion
(per-stage `## Bounded judgment` sections, a workflow-level
`## Authority envelope`, a corrected provenance pointer, a split-and-
partially-revised `20-implement` delegation section, and a new J0 clause
for `10-triage-and-route`'s straddling-candidate case). The `tdd` branch of
`20-implement`'s delegation is deliberately left partially unrevised,
tracking `tdd`'s own unresolved ICM-R3 dispute rather than silently
picking a side — the same scope boundary `implement`'s own pass drew for
its identical citation, independently re-derived here rather than copied,
and applying with at least equal force since `worker-mission` is one of
`tdd`'s two genuinely independent direct parents, not a second-hand
citation. No `draft/` directory accompanies this record; the remediation
list above is the complete deliverable for a STAND-only content revision.

## Validation evidence

- Source-valid: read every file under `.sergeant/workflows/worker-mission/`
  in full; cross-checked every cited `BU-P7-*` unit against
  `docs/gauntlet/promoted-provenance/worker-mission.md`'s own archived
  quotes; confirmed no citation gap (unlike `diagnose-bug`'s ICM-R2
  history with `direct-implementation`, `worker-mission`'s provenance
  trail is fully archived and was read in full).
- Placement-valid: `worker-mission`'s own PL-4 rung is independently
  re-derived from §5.6 applied directly to the package's current stage
  content and outcome, not assumed from its "published" status. The
  `20-implement` delegation-mechanism defect for the four settled branches
  is derived independently from `docs/icm/convention.md` §4 rule 1,
  checked directly against each of those four delegates' own live,
  settled structure (not merely asserted from `implement`'s parallel
  finding). The `tdd`-branch treatment is independently re-derived from
  `implement`'s precedent method, checked against `worker-mission`'s own
  status as `tdd`'s other direct parent rather than assumed to transfer
  automatically (see "The `tdd` citation" subsection above).
- Authority-valid: the seam-confirmation J0 (`BU-WM-04b`) is preserved and
  made explicit in `20-implement`'s own contract rather than deferred to a
  disputed rehome; the straddling-candidate J0 for `10-triage-and-route`
  (`BU-WM-03`) is a new finding, independently derived by applying
  `deepen-module`'s reviewer's own reasoning to this package's structurally
  identical branching point, not copied from any source that already names
  `worker-mission`. Checked against the risk that any of these corrections
  could narrow authority rather than only clarify it (proposal
  §4.6/§6.9's narrowing-only rule) — none does; each makes an already-
  implicit requirement explicit.
- Structurally valid: confirmed via direct grep that `worker-mission` is
  named as a delegate/consumer only by `dispatch` (dispatches it),
  `code-review` (describes it as a possible caller, though the actual path
  is indirect via `implement`), and the five delegate packages' own ICM-R3
  records; confirmed `.sergeant/workflows/worker-mission/provenance.md`
  does not exist (`BU-WM-11`); confirmed `recover-stalled-worker`'s own
  ICM-R3 pass independently checked and found no missing relationship in
  either direction.
- Execution-valid: **out of scope for this producer pass**, as for every
  other ICM-R3 pass this record read — this adjudication is a
  content/citation/placement review, not a re-run of the package;
  `reference/proposal-icm-r-procedure-authority.md` §9.3's execution-
  validation claims remain to be measured separately.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013` decisions 6-7); it does not self-promote.
