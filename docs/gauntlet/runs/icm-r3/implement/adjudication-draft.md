# Package adjudication: implement

ICM-R3 full-reconciliation pass, package `implement`
(`docs/adr/0013-icm-r0-owner-rulings.md`; method
`reference/proposal-icm-r-procedure-authority.md` §8 — Contract, Inventory,
Harvest, Normalize, Placement classification, Authority classification,
Synthesis; §10.4 ICM-R3 scope, wave 2: `implement`'s delegates
(`tdd` at wave 1, `code-review` at ICM-R2) are already reconciled, so this
pass runs against settled — or, for `tdd`, disputed-but-bounded — delegate
state rather than blind). Record shape: `docs/icm/record-shapes.md` §6.
Producer draft only — independent adversarial review (§8.11 of the
proposal) and Captain's reconcile-and-publish pass (§8.12) have not run.
This record and its `draft/` content are themselves draft; neither is
self-promoting (`docs/adr/0013` decision 6).

This pass read both `docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md`
(producer: REHOME) and `docs/gauntlet/runs/icm-r3/tdd/review.md`
(independent reviewer: DISPUTED, not a reversal to STAND) before forming
its own view of what `implement` itself should do about its `tdd`
delegation. See "Driver and admission boundary" below for that independent
judgment — it does not simply adopt either the producer's REHOME draft or
the reviewer's recommendation; it reaches a package-scoped conclusion of
its own, informed by reading the live `code-review` delegation (settled,
not disputed) side by side with the disputed `tdd` delegation and finding
the same structural defect in both, independent of how the `tdd` dispute
itself resolves.

## Original intention

`.sergeant/workflows/implement/CONTEXT.md`: "Implement a piece of work from
a spec or ticket set, explicit-invocation-only." Candidate **W23** from the
N1 reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`) of
`reference/sergeant-upstream/.agents/skills/implement/SKILL.md`, promoted
per `docs/icm/promotion-spec-2026-08-11.md` with a full behavior-unit
citation trail archived at `docs/gauntlet/promoted-provenance/implement.md`.
The upstream skill's own body: "use the tdd workflow where possible, at
seams pre-agreed for testing" (`BU-P2-052`), then "the code-review
skill/workflow is used to review the work" (`BU-P2-054`) — the package was
authored from the start as a thin two-step orchestration over two other
procedures, not as a package with much independent content of its own.

## Current trigger and outcome

Trigger (`CONTEXT.md`, `index.md`): "Explicitly invoked to implement a
defined piece of work (never auto-loaded)" — explicit-invocation-only per
`BU-P2-051`, mirrored cross-harness per `BU-P3-004`. Outcome as packaged:
two ordinary actor stages (`workflow.toml`: `10-implement-with-tdd`,
`30-review`) — implementation proceeds seam by seam (delegating to `tdd`,
folding the demoted `20-verify` checkpoint as a typecheck/test-suite
helper, `BU-P2-053`), then the change is reviewed (delegating to
`code-review`, folding the demoted `40-commit` checkpoint as the concluding
commit helper, `BU-P2-055`). `30-review`'s output carries `promote`
disposition; it is the workflow's last stage.

As packaged, `implement` is `status: published` under
`.sergeant/workflows/` (`index.md`), listed in the root catalog
(`.sergeant/index.md`) and named as one of `worker-mission/20-implement`'s
five selectable disciplines
(`.sergeant/workflows/worker-mission/20-implement/CONTEXT.md`) — it is both
an independently dispatchable workflow and a delegation target itself.

## Driver and admission boundary

Driver: stage-actor throughout — two ordinary actor stages, no Captain
dialogue and no deterministic/execute stage
(`.sergeant/workflows/implement/workflow.toml`). Admission boundary:
in-work — given an already-defined spec or ticket set, `implement` executes
durably from admission (implementation) through a terminal, merge-relevant
result (a reviewed, committed change), independent of whether Captain
remains present. This is not in dispute: applying §5.6's PL-4 question
directly — "given an already-defined intent, repositories, constraints, and
expected outcome, can Sergeant execute this procedure durably from
admission to a terminal result whether or not the Captain remains
present?" — the answer is yes, and no part of this pass, the `tdd`
producer's pass, or the `tdd` reviewer's pass challenges `implement`'s own
PL-4 rung. **Package identity and driver STAND.**

What is genuinely unsettled is not `implement`'s own rung but the
representation of its two `## Delegation` sections, both of which currently
read (near-verbatim, one substituting the delegate's name):

> "This stage's outcome is produced by running **\<delegate\>** to its own
> completion (context composition today — see `docs/icm/convention.md` §4
> on `@@name` versus true nested-workflow invocation, which does not exist
> yet)."

Checked directly against each delegate's own current, settled state rather
than assumed from this prose:

1. **`code-review` (delegated from `30-review`) is not disputed by anyone.**
   Its ICM-R2 adjudication (`docs/gauntlet/runs/icm-r2/code-review/
   adjudication-draft.md`) landed `STAND` and is live in
   `.sergeant/workflows/code-review/` today — four real actor stages
   (`00-pin-fixed-point`, `10-identify-spec-source`,
   `20-30-parallel-review`, `40-aggregate`), each its own fresh execution,
   plus a full `## Authority envelope` in its `CONTEXT.md` naming explicit
   J0/J5 material ("Whether to skip asking the user for the fixed point
   when none is given — J0"; "Whether to merge or rerank the Standards and
   Spec axes — J5 governing constraint, never merge"). Describing an
   invocation of this package as "context composition" is checked directly
   against §4 rule 1 of `docs/icm/convention.md`: "A `@@name` reference used
   to imply 'and then run that other procedure as a sub-workflow' is a
   violation of scope... that intent must be recorded as an engine-gap
   claim... not smuggled through a context reference." `implement/30-review`
   does not even use `@@name` syntax — its prose is a plainer version of
   exactly the smuggled intent that rule forbids: it names the mechanism as
   "context composition" while describing an outcome ("running code-review
   to its own completion") that context composition structurally cannot
   produce. Pulling `code-review`'s four-stage, authority-enveloped,
   review-independence-bearing procedure into `implement/30-review`'s
   single actor turn would collapse four fresh executions into one and
   destroy the review-independence property `code-review`'s own design
   depends on (its `20-30-parallel-review`/`40-aggregate` split exists
   precisely so the aggregation step does not share the reviewing step's
   context). **This defect is independent of the `tdd` dispute and is
   settled by this pass on its own evidence: the `code-review` delegation's
   current wording is wrong today, not contingent on any pending
   adjudication elsewhere.**
2. **`tdd` (delegated from `10-implement-with-tdd`) carries the identical
   wording but an unsettled underlying placement.** The `tdd` producer
   pass (`docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md`) reached
   REHOME (PL-3, folds wholly into `.sergeant/common/contexts/tdd.md`); its
   independent reviewer (`docs/gauntlet/runs/icm-r3/tdd/review.md`) reached
   DISPUTED — not a reversal to STAND, but a finding that the producer's
   "Alternatives considered" section never weighed the PL-7 engine-gap
   alternative that `docs/icm/record-shapes.md` §5's own canonical worked
   example describes almost verbatim for this exact shape ("Two workflows
   both need to invoke a shared... procedure with its own retry/
   measurement, not just shared text" — `tdd` has two current consumers,
   `implement` and `worker-mission`, each needing its own
   seam-agreement/red-green checkpoints). This pass does not re-adjudicate
   `tdd`'s own placement (out of this pass's assigned package scope,
   exactly as the `tdd` producer correctly declined to wire
   `test-quality.md`'s other four consumers out of *its* scope) — but it
   does independently confirm, from the calling side, that the reviewer's
   underlying concern is well-founded: `10-implement-with-tdd`'s own two
   real, currently-distinct checkpoints inside `tdd`
   (`00-agree-seams`, `10-red-green-cycle`) are exactly the "fresh
   executions with their own retry/measurement" the engine-gap alternative
   names, and `implement`'s own delegation prose already anticipates the
   gap without ever filing the record `record-shapes.md` §5 requires.
3. **Point 1 and point 2 converge on the same underlying defect, seen from
   two directions.** The `tdd` dispute is about whether `tdd`'s *own*
   package should collapse into a shared context; this pass's finding is
   narrower and does not depend on that dispute's outcome: `implement`
   itself, independent of which way any one delegate's placement resolves,
   currently describes "invoke another full, checkpointed procedure to
   completion" as if it were the same kind of thing as "pull in some
   reference prose" — and it is not, for either delegate, today. The
   correct fix is not to pick a side in the `tdd` dispute; it is to stop
   letting `implement`'s own stage contracts assert a mechanism
   (context composition) that neither current delegate's actual structure
   supports, and to file the engine-gap record that has been deferred at
   both call sites since promotion.

Conclusion: `implement`'s own driver and PL-4 rung STAND. Its two
`## Delegation` sections do not — both require revision, but the two
revisions differ in what they can responsibly commit to now: the
`code-review` side can be corrected in full immediately (code-review's
placement is settled); the `tdd` side can only be corrected on the
dispute-independent points (the confirmed hidden J0 dependency,
`BU-TDD-04`) without adopting either disputed disposition as fact. See
"Surviving package design" below.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| `BU-IMPL-01` | `CONTEXT.md`/`index.md` workflow-level trigger, purpose, explicit-invocation-only (`BU-P2-050`, `BU-P2-051`, `BU-P3-004` per `docs/gauntlet/promoted-provenance/implement.md`) | PL-4 (§5.6 — admission-to-terminal-result procedure; not disputed, see "Driver and admission boundary") | J5 — explicit-invocation-only is a governing workflow prohibition ("must never be auto-loaded merely because the task looks like implementation"), not a J2 choice any loader may waive | STAND | `draft/CONTEXT.md` Purpose/Trigger (content unchanged; also now stated as a `## Authority envelope` "may not decide" item, see `BU-IMPL-07`) |
| `BU-IMPL-02` | `10-implement-with-tdd/CONTEXT.md` `## Delegation` — delegates to `tdd` (`BU-P2-052`) | Rung inherited from `tdd`'s own disputed placement — **not resolved by this pass** (out of scope; see point 2 above) | J0 the loading stage must honor regardless of `tdd`'s eventual rung: `BU-TDD-04` (`docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md`/`review.md`, CONFIRMED by the independent reviewer) — no test is written at an unconfirmed seam, and this stage's own contract currently never states that, only "context composition" prose that requires reading `tdd`'s full source to discover it | FOLD (revise prose; do not adopt the disputed REHOME fold-notes, do not delete `tdd` as a reference) | `draft/10-implement-with-tdd/CONTEXT.md` |
| `BU-IMPL-03` | Helper: verify, folded from demoted `20-verify` (`BU-P2-053`, N1 adjudication A4) | PL-6 (deterministic helper) | J1 — mechanical: typecheck/single-test-file regularly, full suite once at the end | STAND | unchanged, already correctly folded per `docs/gauntlet/promoted-provenance/implement.md` "Adjudication A4" |
| `BU-IMPL-04` | `30-review/CONTEXT.md` `## Delegation` — delegates to `code-review` (`BU-P2-054`) | PL-7 candidate for the *invocation mechanism* (see engine-gap claim below); `code-review` itself is settled PL-4/STAND, not in dispute | J5 inherited from `code-review`'s own settled Authority envelope (never merge/rerank axes; ask user if no fixed point/spec source) — need not be *restated* here because, unlike `tdd`, the corrected mechanism keeps `code-review` a separately-admitted execution that carries its own envelope intact, per point 3 above | FOLD (revise prose; correct the mechanism description) | `draft/30-review/CONTEXT.md` |
| `BU-IMPL-05` | Helper: commit, folded from demoted `40-commit` (`BU-P2-055`, N1 adjudication A4) | PL-6 | J1/J2 — committing itself is mechanical; message content is a J2 local choice | STAND | unchanged, already correctly folded |
| `BU-IMPL-06` | Both stages' `## Judgment required` boilerplate ("this is an actor stage... inspect evidence, choose among alternatives...", identical word-for-word across both stages) | N/A (authoring-format compliance) | J5 — `docs/icm/convention.md` §6.1 / `docs/adr/0013` decision 4 require a named `## Bounded judgment` section (J2/J1/J0 + completion boundary + decision evidence) on every actor stage, "always present, even when it is only 'inherits workflow envelope unchanged'" | FOLD (replace) | `draft/10-implement-with-tdd/CONTEXT.md`, `draft/30-review/CONTEXT.md` |
| `BU-IMPL-07` | Workflow `CONTEXT.md` has no `## Authority envelope` section | N/A | J5 — `docs/icm/convention.md` §6.1 requires one on every workflow Layer-1 `CONTEXT.md` | FOLD (add) | `draft/CONTEXT.md` |
| `BU-IMPL-08` | `CONTEXT.md` "## Provenance" says "See `provenance.md`" — no such file exists under `.sergeant/workflows/implement/`; the real file is `docs/gauntlet/promoted-provenance/implement.md` (confirmed: `ls .sergeant/workflows/implement/` lists only `10-implement-with-tdd/`, `30-review/`, `CONTEXT.md`, `index.md`, `workflow.toml`) | N/A (structural — broken self-reference) | n/a | FOLD (correct pointer) | `draft/CONTEXT.md`, same defect class as `code-review`'s own ICM-R2 finding #3 ("Broken self-reference") |
| `BU-IMPL-09` | Structural, cross-package: `implement`'s two delegations both currently describe "invoke another full, checkpointed workflow to completion" as "context composition today... which does not exist yet" — a hedge repeated at every call site since promotion (also present, identically worded, in `worker-mission/20-implement/CONTEXT.md`) without ever being filed as `record-shapes.md` §5's required engine-gap record. `code-review` has two live current parents today (`implement`, and `worker-mission` indirectly via `implement`'s own dispatch — confirmed by grep: only `.sergeant/workflows/implement/30-review/CONTEXT.md` names `code-review` as a direct delegate; `worker-mission` reaches it only by selecting `implement` as one of its five disciplines) — this is short of two *independent* direct parents, so the record-shapes.md §5 worked example's literal "two workflows both need to invoke" framing is not yet fully instantiated for `code-review`, but the same shape recurs for `tdd` (two genuinely independent direct parents: `implement` and `worker-mission`) | PL-7 (engine gap) | n/a | new artifact — engine-gap claim | `draft/engine-gap-nested-workflow-invocation.md` |

## Surviving package design

Two actor stages, unchanged in count, order, and identity —
`10-implement-with-tdd` then `30-review`, both `driver: stage-actor`,
`implement` remains PL-4/STAND. Both gain a proper `## Bounded judgment`
section (`BU-IMPL-06`) and the workflow `CONTEXT.md` gains
`## Authority envelope` (`BU-IMPL-07`) and a corrected provenance pointer
(`BU-IMPL-08`), mirroring the same internal-fidelity fixes the `code-review`
ICM-R2 pass applied to itself.

1. **`10-implement-with-tdd`** (`draft/10-implement-with-tdd/CONTEXT.md`):
   content is revised *only* on the dispute-independent point. The
   `## Delegation` prose is kept (still names `tdd` as the technique this
   stage applies, still records that the current mechanism is "context
   composition today" pending a resolved engine-gap or a resolved `tdd`
   placement) but a new `## Bounded judgment` section is added stating the
   seam-confirmation requirement as an explicit J0 the stage itself must
   honor — sourced directly from `tdd/00-agree-seams/CONTEXT.md`'s own
   content (independently re-read by this pass, not copied from the
   disputed `tdd` REHOME draft's `@@tdd` fold notes, since those presuppose
   an accepted disposition this pass does not adopt). A tracking note
   points at the open `tdd` dispute by file path so a future pass does not
   need to rediscover it. **This stage's content should be revised again
   once `tdd`'s own ICM-R3 dispute resolves** (whichever way) — this draft
   is deliberately not final on that point, per this pass's own scope
   boundary (adjudicating `implement`, not re-adjudicating `tdd`).
2. **`30-review`** (`draft/30-review/CONTEXT.md`): content is revised in
   full — `code-review`'s placement is settled, so there is no reason to
   hedge. The `## Delegation` section is renamed `## Delegates to` and
   corrected to state plainly that `code-review` is a separately-admitted
   PL-4 workflow, that today's best-available mechanism for invoking it to
   completion is dispatching it as its own Work (not context pull-in —
   `proposal-next-iteration-icm-workflows.md` §7.7: "An agent can... submit
   another `sgt run`," which, while it loses parent/child trajectory
   identity, at minimum preserves `code-review`'s own four checkpoints and
   authority envelope, unlike context composition which would destroy
   them), and that a true nested-workflow capability (this pass's filed
   engine-gap claim) is the eventual correct fix. A new
   `## Bounded judgment` section is added.
3. **New artifact, not a stage:** `draft/engine-gap-nested-workflow-
   invocation.md`, the full six-field `record-shapes.md` §5 template this
   pattern has been missing since promotion. This is evidence for the
   reference corpus's `engine-pressure.md` per §5's rule 6 — it is not
   itself authorization for engine work, and this pass does not propose
   any `src/` change (hard boundary, `docs/adr/0013` decision 10, runtime
   freeze through ICM-R4).

Nothing in `implement`'s own package is REHOMEd, SPLIT, HARVESTed away, or
RETIREd. The two demoted-and-folded helpers (`BU-IMPL-03`, `BU-IMPL-05`)
remain exactly as N1 adjudication A4 left them; this pass found no defect
in either.

## Inputs and outputs

Inputs consumed by this adjudication: every file under
`.sergeant/workflows/implement/` (`CONTEXT.md`, `index.md`,
`workflow.toml`, both stage `CONTEXT.md`/`output/README.md` pairs);
`docs/gauntlet/promoted-provenance/implement.md`; the upstream source
(`reference/sergeant-upstream/.agents/skills/implement/SKILL.md`); both
current delegates read in full
(`.sergeant/workflows/tdd/CONTEXT.md` and both its stage `CONTEXT.md`
files; `.sergeant/workflows/code-review/CONTEXT.md`, `workflow.toml`, and
`00-pin-fixed-point/CONTEXT.md` to confirm current stage-format state);
`docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md` and
`docs/gauntlet/runs/icm-r3/tdd/review.md` (both read in full, per this
pass's explicit dispatch instruction); `docs/gauntlet/runs/icm-r2/
code-review/adjudication-draft.md`; `.sergeant/workflows/worker-mission/
CONTEXT.md` and `20-implement/CONTEXT.md` (grepped for `implement`,
`tdd`, `code-review` to establish the actual current consumer graph);
`reference/proposal-icm-r-procedure-authority.md` §5, §6, §8, §10.4;
`reference/proposal-next-iteration-icm-workflows.md` §7.5-§7.7;
`.sergeant/common/contexts/bounded-judgment.md`; `docs/icm/convention.md`
§§1, 1a, 2a, 4, 5, 6; `docs/icm/record-shapes.md` §§1a, 5, 6;
`docs/adr/0013-icm-r0-owner-rulings.md`.

Output: this record, plus `draft/CONTEXT.md`, `draft/
10-implement-with-tdd/CONTEXT.md`, `draft/30-review/CONTEXT.md`, and
`draft/engine-gap-nested-workflow-invocation.md` — none of them live edits
(`docs/adr/0013` decision 6, promotable-only review).

## Review and promotion policy

Artifact class: package-adjudication record plus draft revised content and
one engine-gap claim. Draft location: `docs/gauntlet/runs/icm-r3/
implement/` (this record) and its `draft/` subdirectory. Independent
reviewer: this ICM-R3 wave's own review step (a fresh execution, explicit
inputs, review-only contract, no edit authority over this record, per
`docs/adr/0013` decision 7). Acceptance criteria: the "code-review
delegation is wrong today, independent of the `tdd` dispute" argument
holds up against adversarial challenge; the decision to defer
`10-implement-with-tdd`'s full revision until `tdd`'s own dispute resolves,
rather than committing to one side now, is itself defensible (the
alternative — silently adopting REHOME's disputed fold notes — was
rejected, see "Alternatives considered"); the engine-gap claim meets
`record-shapes.md` §5's six-field bar and is not rejected at lint; no
citation is fabricated. Promotion action: Captain's reconcile-and-publish
pass (§8.12) accepts/rejects/merges/parks each finding; if the `30-review`
revision is accepted it lands independently of `tdd`'s own resolution (it
does not depend on that dispute); the `10-implement-with-tdd` revision as
drafted here may land now (it only adds the confirmed J0 restatement) but
is explicitly flagged as needing a follow-up revision once `tdd` resolves.
Failure/remediation: if review finds the "code-review delegation is wrong
today" argument does not hold (e.g., a reviewer judges "context
composition" an acceptable informal gloss for "dispatch as a separate
Work" and not the smuggled-nested-workflow anti-pattern this record treats
it as), that specific finding is dropped and the rest of this record is
unaffected — the other findings (`BU-IMPL-06` through `BU-IMPL-09`) do not
depend on it.

## Alternatives considered

- **Wait for `tdd`'s dispute to resolve before adjudicating `implement` at
  all.** Rejected: this pass's dispatch instruction explicitly requires
  reading both the `tdd` producer draft and its reviewer's dispute and
  forming an independent judgment now, including "whether the dispute
  changes what implement itself should do" — deferring the entire package
  would not answer that question. The package-scoped finding this pass
  reaches (`BU-IMPL-09`, the shared "context composition" mischaracterization
  defect) turned out to be answerable without waiting, since it rests on
  `code-review`'s settled state, not on how `tdd`'s dispute resolves.
- **Adopt the `tdd` producer's REHOME fold notes verbatim for
  `10-implement-with-tdd`** (`docs/gauntlet/runs/icm-r3/tdd/draft/
  fold-notes/implement-10-implement-with-tdd.md` already drafts this exact
  edit). Rejected: that fold note is explicitly conditioned on REHOME being
  accepted ("This REHOME must not be promoted before, or independently of,
  the `implement`/`worker-mission` fold" — the `tdd` record's own words),
  and the independent reviewer disputed REHOME, not confirmed it. Promoting
  content that presupposes `@@tdd` exists, when `.sergeant/common/
  contexts/tdd.md` is itself unpublished disputed draft content, would
  strand `implement`'s own stage on a reference that may never be admitted
  in that form. This pass instead makes only the move that is correct
  under every disputed outcome: state the seam-confirmation J0 explicitly,
  in this stage's own contract, independent of where `tdd`'s content
  eventually lives.
- **Leave both `## Delegation` sections untouched and treat "context
  composition today... does not exist yet" as sufficient self-disclosure
  of the gap.** Rejected: `docs/icm/convention.md` §4 rule 1 is explicit
  that this exact hedge, when it is actually being used to mean "run this
  as a sub-workflow," must be recorded as a real engine-gap claim, not left
  as a standing caveat repeated at every call site. Three packages
  (`implement` twice, `worker-mission` once) currently carry the identical
  unaddressed hedge; this pass files the record rather than adding a
  fourth instance of the same deferral.
- **File the engine-gap claim but change nothing else about the stage
  prose.** Rejected: a claim on file does not fix the immediate,
  independently-actionable defects — the missing `## Bounded judgment`
  sections, the missing workflow `## Authority envelope`, the broken
  `provenance.md` pointer, and (for `code-review` specifically, since its
  placement is not in dispute) the mischaracterized invocation mechanism.
  Those are fixed now; only the deeper runtime capability question is
  deferred to the filed claim, per `docs/adr/0013` decision 10's runtime
  freeze.
- **REHOME or SPLIT `implement` itself** (e.g., treating its thin,
  mostly-delegated content as evidence it should not be its own workflow).
  Rejected: `implement`'s own PL-4 admission-to-terminal-result test holds
  cleanly (see "Driver and admission boundary") and is not contested by any
  source this pass read. A workflow being *thin* — most of its behavior
  delegated elsewhere — is not itself a placement defect; `worker-mission`
  is built the same way and is not disputed on that basis either.

## Final disposition
STAND

Package identity, driver, and PL-4 rung are correct and not in dispute —
see the Behavior-unit dispositions table for the individual FOLD grain
within this STAND. Internal restructuring is required before promotion
(per-stage `## Bounded judgment` sections, a workflow-level
`## Authority envelope`, a corrected provenance pointer, and a corrected
`code-review` delegation mechanism), plus a new engine-gap claim filed
against a defect this pass independently confirmed recurs across all three
of this package's own delegation sites. The `tdd`-delegation side of
`10-implement-with-tdd` is deliberately left partially unrevised, tracking
`tdd`'s own unresolved ICM-R3 dispute rather than silently picking a side.
Proposal §8.9 requires draft treatment for any "generated or substantially
rewritten" package regardless of its top-level modifier, so the corrected
content is written under `docs/gauntlet/runs/icm-r3/implement/draft/`
rather than edited in place.

## Validation evidence

- Source-valid: read every file under `.sergeant/workflows/implement/` in
  full; cross-checked every cited `BU-P2-*`/`BU-P3-*` unit against
  `docs/gauntlet/promoted-provenance/implement.md`'s own archived quotes;
  read the upstream `SKILL.md` directly.
- Placement-valid: `implement`'s own PL-4 rung is independently re-derived
  from §5.6 applied directly to the package's current stage content and
  outcome, not assumed from its "published" status. The delegation-
  mechanism defect (`BU-IMPL-04`, `BU-IMPL-09`) is derived independently
  from `docs/icm/convention.md` §4 rule 1 and `proposal-next-iteration-icm-
  workflows.md` §7.7, checked directly against `code-review`'s own live,
  settled, four-stage structure rather than asserted from the dispatch
  instruction's framing.
- Authority-valid: the seam-confirmation J0 (`BU-IMPL-02`) is preserved
  and made explicit in this stage's own contract rather than deferred to a
  disputed rehome; `code-review`'s own settled Authority envelope is
  checked directly (`.sergeant/workflows/code-review/CONTEXT.md`) and
  confirmed to remain intact under the corrected invocation description
  (point 3, "Driver and admission boundary") rather than silently narrowed
  — checked against the risk that a mechanism correction could narrow
  authority (proposal §4.6/§6.9's narrowing-only rule).
- Structurally valid: confirmed via direct grep that `code-review` is
  named as a delegate only by `.sergeant/workflows/tdd/CONTEXT.md`
  (descriptively, not as a delegation) and
  `.sergeant/workflows/implement/30-review/CONTEXT.md` (the actual
  delegation this pass revises); confirmed `worker-mission/20-implement`
  reaches `code-review` only indirectly, via selecting `implement` as one
  of its five disciplines, not as a direct second parent; confirmed
  `.sergeant/workflows/implement/provenance.md` does not exist
  (`BU-IMPL-08`).
- Execution-valid: **out of scope for this producer pass**, as for the
  `tdd` pass it follows — this adjudication is a content/citation/
  placement review, not a re-run of the package;
  `reference/proposal-icm-r-procedure-authority.md` §9.3's execution-
  validation claims remain to be measured separately.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013` decisions 6-7); it does not self-promote.
