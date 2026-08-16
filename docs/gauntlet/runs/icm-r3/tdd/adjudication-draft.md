# Package adjudication: tdd

ICM-R3 full-reconciliation pass, package `tdd`
(`docs/adr/0013-icm-r0-owner-rulings.md`; method
`reference/proposal-icm-r-procedure-authority.md` §8 — Contract, Inventory,
Harvest, Normalize, Placement classification, Authority classification,
Synthesis; §10.4 ICM-R3 scope). Record shape:
`docs/icm/record-shapes.md` §6. Producer draft only — independent
adversarial review (§8.11 of the proposal) and Captain's
reconcile-and-publish pass (§8.12) have not run. This record and its
`draft/` content are themselves draft; neither is self-promoting
(ADR 0013 decision 6).

Dispatch context named `tdd` as "Likely STAND with Authority-envelope/
Bounded-judgment additions per the now-established pattern." That
hypothesis is **not confirmed** by this pass — see "Driver and admission
boundary" below. The dispatch instruction itself required verifying the
hint against current content rather than assuming it; this record does
that and reaches a different disposition.

## Original intention

`.sergeant/workflows/tdd/CONTEXT.md`: "Test-driven development for one
confirmed seam at a time: red, green, one minimal implementation."
Candidate **W22** from the N1 reference-corpus decomposition
(`docs/gauntlet/contracts/N1.md`) of
`reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` (plus its
co-located `tests.md` and `mocking.md`), promoted per
`docs/icm/promotion-spec-2026-08-11.md` with a full behavior-unit citation
trail archived at `docs/gauntlet/promoted-provenance/tdd.md`. The
package's own reviewer notes already record that most of the upstream
source is reference guidance rather than procedure: "16 units land in the
`test-quality` shared context, not in this workflow"
(`.sergeant/workflows/tdd/CONTEXT.md` "Notes for reviewers").

## Current trigger and outcome

Trigger (`CONTEXT.md`, `index.md`): "A feature or bug fix is being
implemented test-first." Outcome as packaged: two ordinary actor stages
(`workflow.toml`: `00-agree-seams`, `10-red-green-cycle`) — seams written
down and confirmed with the user, then one seam/one test/one minimal
implementation per cycle, vertical slices only, with refactoring
explicitly excluded and handed to `code-review`/`deepen-module`.

As packaged, `tdd` is `status: published` under `.sergeant/workflows/`
(`index.md`), listed in the root catalog (`.sergeant/index.md` line 39)
and in `AGENTS.md`'s list of "published workflows" the corpus rewrite
consumed content into (line 233) — every catalog surface currently treats
it as an independently dispatchable workflow, identical in kind to
`diagnose-bug` or `code-review`.

## Driver and admission boundary

As packaged: driver is stated implicitly as `actor-stage (§6.4, judgment)`
for both stages, admission boundary **in-work**, reached (per every other
published workflow) through `sgt run --workflow tdd` or workflow-selection
fallback.

This does not survive inspection of the package's own actual current
content and its actual current consumers, checked directly rather than
assumed from the dispatch hint:

1. **The proposal's own Placement Ladder names this exact behavior as its
   first PL-3 example.** §5.5 ("Actor skill or shared method"), Examples:
   "a TDD technique; a document-critique rubric; threat modeling; ..." —
   listed *before* any of the other examples. This is not incidental
   phrasing; §5.5's Discriminator from PL-4 states plainly: "An actor
   skill does not independently own one admitted intent from start to
   terminal Work outcome." `tdd` does not: it has no intent of its own to
   own — its two stages exist only in service of whatever intent the
   surrounding implementation Work already carries. (Corrected 2026-08-16,
   ICM-R3 review: the earlier text here cited `10-red-green-cycle`'s
   `promote` artifact disposition as evidence *against* a terminal
   outcome — misread; `promote` means the artifact *does* survive into the
   merge. The narrower, accurate gap the source note actually flags is
   authoring, not placement: no dedicated finalize *stage* curates that
   promotion mechanically, a human does it at merge time — irrelevant to
   whether `tdd` is a workflow, since plenty of correctly-PL-4/5 packages
   share that same authoring gap.)
2. **`tdd` is never invoked directly by any current package.** Grepped
   every workflow and skill under `.sergeant/workflows/` and `skills/`:
   the only two references to `tdd` as something "run" are
   `.sergeant/workflows/implement/10-implement-with-tdd/CONTEXT.md`
   ("This stage's outcome is produced by running **tdd** to its own
   completion") and `.sergeant/workflows/worker-mission/20-implement/
   CONTEXT.md` (same phrasing, one of five named disciplines
   `10-triage-and-route` may select). Both delegations are, by their own
   stated wording, "context composition today — see `docs/icm/
   convention.md` §4 on `@@name` versus true nested-workflow invocation,
   which does not exist yet" — i.e., today's actual mechanism for
   "running tdd" is reading its content into the caller's own stage
   context, not dispatching a separate admitted Work. That is exactly the
   PL-3 destination (§5.5: "A skill, workflow-local reference, or
   `.sergeant/common/` shared context") already in effect informally,
   under a PL-4 label.
3. **Corrected 2026-08-16, ICM-R3 review + owner ruling: the original
   argument here (§2a's "does the same thing every time" test) is
   withdrawn — as written it would disqualify every admitted workflow,
   since all of them fix procedure while varying content
   (`diagnose-bug` always reproduces/isolates/proves/remediates
   regardless of which bug; that doesn't make it PL-3).** The actual
   discriminator, put directly by the owner: `tdd` is a *technique* —
   something an actor applies while it is already doing implementation
   work — not a separate undertaking a caller hands off to and waits on.
   Its current two-stage, fresh-execution shape is an artifact of how it
   was extracted from the upstream corpus at N1, not evidence the
   technique itself needs independent checkpoint/retry semantics. A
   confirmed seam and a red-green cycle are not things one *dispatches
   to*; they are how an actor conducts the implementation turn it is
   already in.
4. **`shared` is the correct modifier, not `local`** (proposal §5.10): two
   independent current consumers (`implement`, `worker-mission`) already
   apply the identical contract (same trigger — implementing test-first;
   same stages — agree seams, then red-green cycle). §5.5's Destination
   list resolves this to `.sergeant/common/` shared context, matching how
   the corpus already treats other reusable techniques
   (`@@codebase-design-vocabulary`, `@@domain-modeling`, and the
   already-planned-but-never-built `@@test-quality`, below).

Conclusion: driver is **stage-actor** (an actor loads and applies this
technique inside its own stage, exactly as PL-3's definition describes),
not a Sergeant-driven durable Work of its own. Admission boundary:
**always** — the technique applies whenever an actor stage is doing
test-first implementation, independent of any Work-admission boundary of
its own.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-TDD-01 | `CONTEXT.md`/`index.md` — workflow-level trigger and purpose (`BU-P2-104`, `BU-P2-105` per `docs/gauntlet/promoted-provenance/tdd.md`) | PL-3 (actor skill / shared method, §5.5 — not PL-4; see "Driver and admission boundary" above) | J2 — contributed to the loading stage, not claimed independently (`docs/icm/convention.md` §7.4) | REHOME | `.sergeant/common/contexts/tdd.md` (opening paragraph) |
| BU-TDD-02 | `00-agree-seams/CONTEXT.md` — seam agreement required before any test; the concrete elicitation question (`BU-P2-109`, `BU-P2-110`) | PL-3 | **J0 the loading stage must honor**: no test is written at an unconfirmed seam — this is a stop-and-ask requirement on the *caller*, not a J2 choice the caller may skip or infer | REHOME | `.sergeant/common/contexts/tdd.md` ("Seams" section) |
| BU-TDD-03 | `10-red-green-cycle/CONTEXT.md` — vertical slicing, red-before-green, one-slice-at-a-time (`BU-P2-113`, `BU-P2-114`, `BU-P2-115`) | PL-3 | J2 — which concrete seam/idiom/minimal-implementation shape to choose within one confirmed cycle; J1 — ordering of otherwise-equivalent confirmed seams | REHOME | `.sergeant/common/contexts/tdd.md` ("Rules of the loop" section) |
| BU-TDD-04 | Cross-package: `implement/10-implement-with-tdd/CONTEXT.md` and `worker-mission/20-implement/CONTEXT.md`, both delegating to `tdd` via undeclared context composition, neither restating the seam-confirmation J0 requirement in its own contract | N/A — an authority-envelope completeness gap in the *consuming* stages, not a placement question for `tdd` itself | J0 (`BU-TDD-02`'s requirement is currently invisible except by reading the delegated package's own stage in full — a hidden contract-bearing dependency, `docs/icm/record-shapes.md` §1a rule 4 / `convention.md` §1a rule 1) | **FOLD** (finding recorded here; the actual edit belongs to a later pass over `implement`/`worker-mission`, out of this pass's assigned scope) | draft fold notes at `draft/fold-notes/implement-10-implement-with-tdd.md` and `draft/fold-notes/worker-mission-20-implement.md` |
| BU-TDD-05 | `CONTEXT.md` "Notes for reviewers" — "16 units land in the `test-quality` shared context, not in this workflow"; confirmed against `reference-corpus/shared-context-map.md` (`test-quality`, `BU-P2-106,107,108,111,112,117`-`127`, shared by `diagnose-bug`, `prototype`, `tdd`, `implement`, `deepen-module`) | PL-3 | J2 — reference guidance an actor consults, contributes no J0 of its own (unlike `BU-TDD-02`) | **gap, not yet a placement question this package can resolve alone** — `.sergeant/common/contexts/test-quality.md` does not exist; grepped `.sergeant/common/` directly, only `bounded-judgment.md` is present; no `@@test-quality` reference exists anywhere in `.sergeant/` | drafted alongside this pass at `draft/.sergeant/common/contexts/test-quality.md`, since its entire source is this package's own upstream material (`tests.md`, `mocking.md`), but wiring its five named consumers is out of this pass's scope — see that file's own header |
| BU-TDD-06 | Every stage `CONTEXT.md` in this package uses `## Judgment required` boilerplate, not the `## Bounded judgment` heading with named J2/J1/J0 subsections `docs/icm/convention.md` §6.1 / ADR 0013 decision 4 require of every actor stage | N/A (authoring-format compliance) | J5 (governing requirement this package predates) | moot under REHOME — a shared context does not carry a per-stage Bounded-judgment section of its own (§7.4); its J-contributions are stated in prose instead, as done above in `draft/.sergeant/common/contexts/tdd.md`'s own "What this context contributes" section | n/a |

## Surviving package design

No stage, Captain skill, or new workflow-local content survives at PL-4 or
PL-5. `tdd`'s entire content — the loop discipline (`BU-TDD-01`–`03`) and
the previously-unbuilt reference-guidance half (`BU-TDD-05`) — relocates
to two `.sergeant/common/` shared contexts:

1. **`.sergeant/common/contexts/tdd.md`** (`@@tdd`) — the red-green
   discipline and seam-agreement requirement, drafted in full at
   `draft/.sergeant/common/contexts/tdd.md`. Carries an explicit "What
   this context contributes when loaded inside a stage" section stating
   its J0/J2/J1 contributions, since a shared context has no
   Bounded-judgment section of its own (`convention.md` §7.4) but the
   seam-confirmation requirement (`BU-TDD-02`) is exactly the kind of
   material decision the ladder does not let disappear silently on
   rehome.
2. **`.sergeant/common/contexts/test-quality.md`** (`@@test-quality`) —
   the reference-guidance half this package's own notes already promised
   but that was never materialized, drafted at
   `draft/.sergeant/common/contexts/test-quality.md`. This closes
   `BU-TDD-05`'s gap for the `tdd` half of its intended use; wiring its
   other four named consumers (`diagnose-bug`, `prototype`, `implement`,
   `deepen-module`) is explicitly left to those packages' own future
   ICM-R3 passes, not silently done here.

`.sergeant/workflows/tdd/` itself does not survive: on promotion it is
deleted in full (`workflow.toml`, `index.md`, `CONTEXT.md`, both stage
directories and their `output/README.md` files), following the same
precedent already set by `sergeant-help` and `grilling`'s prior
retirements (both fully removed from `.sergeant/workflows/` once ported;
cited directly in the ICM-R2 `direct-implementation` adjudication,
`docs/gauntlet/runs/icm-r2/direct-implementation/adjudication-draft.md`).
The `implement` and `worker-mission` consuming stages are not edited by
this pass (out of its assigned package scope) — `BU-TDD-04`'s fold notes
record the concrete diff for whoever's ICM-R3 pass covers those two
packages next, per §10.4's instruction to run in bounded waves organized
by delegation cluster so names never point at deleted or rehomed
identities. **This REHOME must not be promoted before, or independently
of, the `implement`/`worker-mission` fold** — if `.sergeant/workflows/tdd/`
is deleted while those two packages' `CONTEXT.md` files still say "running
**tdd** to its own completion," both delegations point at a deleted
identity, which `docs/icm/convention.md` §1 rule 3's uniqueness
requirement and this pass's own citation discipline both treat as a
violation.

## Inputs and outputs

Inputs consumed by this adjudication: every file under
`.sergeant/workflows/tdd/` (`CONTEXT.md`, `index.md`, `workflow.toml`,
both stage `CONTEXT.md`/`output/README.md` pairs);
`docs/gauntlet/promoted-provenance/tdd.md`; the upstream source
(`reference/sergeant-upstream/.agents/skills/tdd/SKILL.md`, `tests.md`,
`mocking.md`); every current delegator, read in full
(`.sergeant/workflows/implement/CONTEXT.md`,
`10-implement-with-tdd/CONTEXT.md`; `.sergeant/workflows/worker-mission/
CONTEXT.md`, `20-implement/CONTEXT.md`); `reference-corpus/
shared-context-map.md`'s `test-quality` entry;
`reference/proposal-icm-r-procedure-authority.md` §5, §6, §8;
`.sergeant/common/contexts/bounded-judgment.md`; `docs/icm/convention.md`;
`docs/icm/record-shapes.md` §6; `AGENTS.md` (grepped for `tdd`);
`.sergeant/index.md` (grepped for `tdd`). No `provenance.md` gap was found
in this package — unlike the ICM-R2 `direct-implementation` case, `tdd`'s
citation trail is fully archived at `docs/gauntlet/promoted-provenance/
tdd.md` and was read in full, not assumed.

Output: this record, plus `draft/.sergeant/common/contexts/{tdd,
test-quality}.md` (the rehomed shared-context content) and
`draft/fold-notes/*.md` (the consumer-side and catalog-side diffs this
REHOME implies, none of them live edits — `docs/adr/0013` decision 6,
promotable-only review).

## Review and promotion policy

Artifact class: package-adjudication record plus draft rehome content and
fold notes. Draft location: `docs/gauntlet/runs/icm-r3/tdd/` (this record)
and its `draft/` subdirectory. Independent reviewer: this ICM-R3 wave's
own review step (a later, independently-executed stage per `docs/adr/0013`
decision 7 — fresh execution, explicit inputs, review-only contract, no
edit authority over this record). Acceptance criteria: the PL-3 argument
in "Driver and admission boundary" holds up against adversarial challenge
(in particular, whether the proposal's own PL-3 example list is being read
correctly and whether "context composition today" genuinely establishes
current PL-3-shaped behavior rather than merely aspirational wording); no
citation is fabricated; the `implement`/`worker-mission` fold notes are
complete enough that promoting this REHOME does not strand a dangling
`tdd` reference. Promotion action: Captain's reconcile-and-publish pass
(§8.12) accepts/rejects/merges/parks each finding, then — only if accepted,
and only in the same change or a coordinated one that also lands the
`implement`/`worker-mission` fold — deletes
`.sergeant/workflows/tdd/`, adds `.sergeant/common/contexts/tdd.md`, adds
`.sergeant/common/contexts/test-quality.md`, updates `.sergeant/index.md`
and `AGENTS.md` per `draft/fold-notes/catalog-updates.md`, and updates the
two consuming stages per their own fold notes. Failure/remediation: if
review finds the PL-3 argument does not hold (e.g., a future nested-
workflow invocation capability makes an independently dispatchable `tdd`
Work coherent, resolving the §5.5 discriminator differently), the package
reverts toward `STAND` with the existing Bounded-judgment/Authority-
envelope additions the original dispatch hint anticipated, and this record
is superseded, not silently edited.

## Alternatives considered

- **STAND, with Authority-envelope and Bounded-judgment additions** (the
  dispatch hint). Rejected as this record's disposition: the package fails
  its own PL-4 discriminator once actually checked against §5.5 — no
  stage produces a terminal, independently meaningful outcome, and the
  proposal's own text names this exact behavior class as PL-3's lead
  example. Adding an Authority envelope to a package that should not be a
  workflow at all treats the symptom (missing required section) while
  leaving the actual placement defect in place.
- **SPLIT: keep `00-agree-seams` as a real checkpoint (its own stage,
  reachable independently) and fold only `10-red-green-cycle` into a
  shared context.** Rejected: `00-agree-seams` also fails the PL-5
  reimplementation test (§5.7) — "if the current mechanism were replaced
  tomorrow, would operators still care that the procedure entered,
  blocked in, retried, completed, or failed at this boundary?" — nothing
  about seam confirmation needs an independent fresh-execution checkpoint
  separate from the implementation stage that immediately follows it in
  the same conversation; it needs the caller to actually ask the question,
  which a J0 clause in the loading stage's own Bounded-judgment section
  achieves without a durable checkpoint of its own.
- **HARVEST into `implement` alone** (the workflow with the closest name
  match), leaving `worker-mission`'s delegation dangling. Rejected: two
  current consumers share the identical contract (§5.10's own test for
  "shared" vs. "local"); harvesting into one consumer's private surface
  would just recreate the duplication `.sergeant/common/` exists to
  prevent, and would still leave `worker-mission`'s reference broken.
- **Leave `.sergeant/workflows/tdd/` in place as a residual "documentation
  only" directory rather than deleting it.** Rejected: `docs/icm/
  convention.md` §2 rule 2 treats a package existing identically (or, here,
  vestigially) in two places as a violation — "it means the boundary was
  not actually crossed, only copied." A deleted-with-provenance directory,
  not a frozen stub, is the correct outcome once the shared context is the
  admitted surface.
- **Build `test-quality.md` in full including wiring all five named
  consumers now**, since this pass already read the whole upstream source.
  Rejected as exceeding this pass's assigned surface: the dispatch scoped
  this producer to the `tdd` package specifically; editing `diagnose-bug`,
  `prototype`, `deepen-module` (none inventoried in this pass) to add
  `@@test-quality` references would be exactly the "search for, infer, or
  relocate a write target outside the stage's own assigned worktree"
  pattern this session's own J0 guardrail (`00-investigate` stage contract,
  citing `docs/adr/0013` decision 8's B9 finding) exists to prevent, one
  level up — writing into another package's assigned scope instead of
  stopping at this package's boundary. The shared context itself is
  drafted (its content is squarely `tdd`'s own source material); wiring
  other packages to it is not.

- **PL-7 engine gap: file for real nested-workflow invocation so `tdd`'s
  two stages keep independent fresh-execution and per-seam retry/
  measurement, rather than collapsing into shared text a caller loads
  inline.** Added 2026-08-16 per the ICM-R3 independent reviewer's
  finding that this alternative was required to be weighed and was not
  (`record-shapes.md` §5.9, §4 rule 1). **Rejected, with an explicit
  mechanics-specific rationale this time, per owner ruling the same
  day:** `tdd` is a technique, not a procedure with its own intent —
  §5.5's own discriminator from PL-4 is exactly this: an actor skill
  "does not independently own one admitted intent from start to terminal
  Work outcome," and `tdd` never has one, it only ever operates inside
  whatever intent the calling implementation Work already carries. The
  per-seam retry/measurement `tdd` has *today* is a byproduct of how the
  N1 extraction happened to draw stage boundaries around upstream prose,
  not a property the technique requires to remain correct — a confirmed
  seam and a red-green cycle are conducted *within* an actor's ongoing
  implementation turn, not handed off to and awaited from a separate
  execution. Losing that stage-level granularity when it folds into
  `@@tdd` is therefore an acceptable, correctly-priced loss for a
  technique this small (1–2 cycles per seam), not evidence Sergeant is
  missing a needed capability. Filing a PL-7 claim here would be
  inventing engine work to preserve boundaries that were never load-
  bearing in the first place — exactly what §4.8's "lowest viable rung"
  rule and the hard runtime freeze (ADR 0013 decision 10) both argue
  against.

## Final disposition
REHOME (confirmed, corrected — see "Driver and admission boundary" and
"Alternatives considered" above for the 2026-08-16 corrections resolving
this package's ICM-R3 reviewer dispute)

## Validation evidence

- Source-valid: read every file under `.sergeant/workflows/tdd/` in full;
  cross-checked every cited `BU-P2-*` unit against
  `docs/gauntlet/promoted-provenance/tdd.md`'s own archived quotes;
  read the upstream `SKILL.md`, `tests.md`, and `mocking.md` directly
  rather than trusting the provenance file's summaries alone.
- Placement-valid: the PL-3-vs-PL-4 argument in "Driver and admission
  boundary" is derived independently from `reference/proposal-icm-r-
  procedure-authority.md` §5.5/§5.6/§2a's execution-surface test, applied
  directly to this package's own current stage content and current
  consumers — not copied from the dispatch hint (which this record
  contradicts) or from the ICM-R2 `direct-implementation` adjudication's
  passing mention of `tdd` as "the separately-admitted `tdd` workflow"
  (that ICM-R2 record did not itself adjudicate `tdd`'s own placement; it
  is evidence of the corpus's prior *assumption*, not a settled finding
  this record needed to defer to).
- Authority-valid: the seam-confirmation J0 requirement (`BU-TDD-02`) is
  explicitly preserved and made more visible by this rehome (stated
  directly in the shared context's own "What this context contributes"
  section, and named as a required addition in both consumer-side fold
  notes) rather than dropped — checked directly against the risk that
  REHOME could silently narrow authority (proposal §4.6/§6.9's
  narrowing-only rule).
- Structurally valid: confirmed via direct grep that no other package
  under `.sergeant/workflows/` or `skills/` references `tdd` beyond the
  two delegators already covered by fold notes; confirmed
  `.sergeant/common/contexts/` currently contains only `bounded-
  judgment.md` (so `@@tdd` and `@@test-quality` do not collide with an
  existing name, per `convention.md` §4 rule 3); confirmed
  `.sergeant/index.md` line 39 and `AGENTS.md` line 233 are the only two
  catalog surfaces naming `tdd` as a published workflow.
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation/placement review, not a re-run of
  the package; `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims remain to be measured separately, and in
  any case would need to be re-targeted at the rehomed shared context's
  two consumers once the fold lands, not at the retired workflow.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013` decisions 6-7); it does not self-promote.
