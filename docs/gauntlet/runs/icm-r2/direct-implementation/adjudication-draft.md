# Package adjudication: direct-implementation

Producer draft, ICM-R2 pilot (`docs/adr/0013-icm-r0-owner-rulings.md`
decisions 8-9), package `direct-implementation`. Method:
`reference/proposal-icm-r-procedure-authority.md` §8 (Contract, Inventory,
Harvest, Normalize, Placement classification, Authority classification,
Synthesis). Record shape: `docs/icm/record-shapes.md` §6. This record is
produced by the reconciliation's producer step; it has not yet passed
independent adversarial review or Captain's reconcile-and-publish pass
(§8.10-8.12 of the proposal) and carries no promotion authority on its own
(`docs/adr/0013` decision 6).

## Original intention

`.sergeant/workflows/direct-implementation/CONTEXT.md`: "Implement in the
current session, against one owning repository, under the same delivery
contract as a dispatched worker." Candidate **W6** from the N1
reference-corpus decomposition of `reference/sergeant-upstream`, promoted
as a five-stage published workflow (`workflow.toml` version 2, `index.md`
`status: published`), decomposed from two overlapping upstream sources —
`AGENTS.md`'s six-stage `direct-mode` description and
`docs/using-sergeant.md`'s eight-step `direct-implementation` procedure
(the package follows the docs' finer boundary; see its own "Notes for
reviewers", Conflict X16).

## Current trigger and outcome

Trigger (`CONTEXT.md`, workflow-level): "The user explicitly asks to work
in this session, and one repository owns the complete outcome." Outcome:
five ordinary actor stages run in sequence —
`01-load-task-context` -> `03-claim-and-implement` (folds demoted
`02-reconcile-existing-state`) -> `04-validate` -> `05-shipping-gate`
(delegates to `validate-and-ship`) -> `06-pr-and-merge` (folds demoted
`07-record-outcomes`) — ending with a merged PR and recorded handoff
state.

## Driver and admission boundary

As packaged: `driver: deterministic-mechanism` is not claimed anywhere;
every stage's own `CONTEXT.md` states `driver` implicitly by declaring
itself an actor stage ("ladder §6.4, judgment"). Admission boundary as
packaged: **in-work**, reached only through `sgt run --workflow
direct-implementation` (or workflow-selection fallback) — i.e., through
Work admission and dispatch, exactly like any other of the 23 published
workflows.

This is the load-bearing tension named in this pilot's own dispatch
instructions, and it survives inspection of the package's actual content,
not just the instruction's hint: the package's own `CONTEXT.md` Purpose
line and Trigger both describe work that happens **because** the user
asked to stay in the current session and avoid dispatch — the same
condition current `AGENTS.md` ("When NOT to use `sgt`") states as exactly
when `sgt run` is **not** used ("Direct, in-session implementation is used
instead"). But the only way this package's own stages execute today is
`sgt run` admitting them as a durable Work — the opposite of staying in
session. Applying PL-4's own required properties (§5.6 of the proposal)
makes the contradiction structural, not just rhetorical: PL-4 requires "a
result that is meaningful independent of the original conversation
continuing" and states "conversation cannot be its primary product" — but
this package's entire reason to exist *is* that the original conversation
continues and owns the outcome. A package cannot coherently be PL-4/PL-5
(workflow/stage, reached by dispatch) while its own trigger names the
condition under which dispatch is explicitly not used. This is also true
of the session producing this very record: an ICM-R2 pilot package
adjudication is itself dispatched Work reviewing whether packaging
"stay in the current session" as dispatched Work makes sense — the same
inversion the package embodies, one level up.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-P1-007 | `reference/sergeant-upstream/AGENTS.md` L22-23 | PL-1 | J5 (routing invariant, binding on Captain) | ABSORBED | `AGENTS.md` "When NOT to use `sgt`" already states this trigger verbatim (BU-0004, BU-0009) |
| BU-P1-016 | `reference/sergeant-upstream/AGENTS.md` L39-41 | PL-1 | J5 (routing invariant) | ABSORBED | `AGENTS.md` "When NOT to use `sgt`" (spanning-repo exclusion) + Guardrails ("standing authorization never extends to skipping the shipping gate...") already cover this |
| BU-P1-107 | `reference/sergeant-upstream/docs/what-is-sergeant.md` L62-66 | PL-1 | J5 (routing invariant) | FOLD | `AGENTS.md` "When NOT to use `sgt`" — the *general* claim ("direct mode is never a lighter path") is currently stated only scoped to sergeant-rs's own code (`AGENTS.md` "Working on sergeant-rs itself", BU-0018/BU-0113/BU-0114); fold a repo-general restatement into the routing section itself |
| BU-P8-055 | `reference/sergeant-upstream/docs/using-sergeant.md` L21-28 | PL-6 (mechanism enumeration) | N/A — mechanism, not a decision | RETIRE | the eight-step enumeration is the obsolete packaging itself; each constituent behavior is dispositioned individually in this table, none needs the ordered list to survive as a structure |
| BU-P1-008 | `reference/sergeant-upstream/AGENTS.md` L24-25 | PL-0 (mechanism obsolete: `sgt-context`/`td context` do not exist in the current CLI or docs — grepped, zero hits outside `reference/`) | J1 (local execution detail, no material decision) | ABSORBED | `AGENTS.md` Standard workflow loop step 1 ("Load estate context... before acting") + `validate-and-ship`'s `00-check-scope` (directly-invoked entry) already own "load context before mutating" |
| BU-P1-009 | `reference/sergeant-upstream/AGENTS.md` L26-27 | PL-2 (Captain-skill judgment: checked once, before committing to a mode, not inside a dispatched stage) | J2 — delegated to Captain: inspect running work before choosing direct mode over dispatch | FOLD | `AGENTS.md` Standard workflow loop step 2 ("Check running work... reuse or resume a matching Work item") does this for the dispatch path only; extend it to state it applies equally before Captain begins direct, in-session implementation |
| BU-P8-056 | `reference/sergeant-upstream/docs/using-sergeant.md` L23 | PL-2 (same behavior, second source) | J2, same grant as BU-P1-009 | FOLD | same destination as BU-P1-009 — duplicate finding from the two overlapping upstream sources (Conflict X16), not two behaviors |
| BU-P1-010 | `reference/sergeant-upstream/AGENTS.md` L28-29 | PL-4 (already-admitted workflow) | J4 (bound Work intent: the task being claimed) | ABSORBED | `validate-and-ship`'s `10-do-the-work` (directly-invoked entry: "the described task is carried out and committed on a feature branch") already owns this; the TDD technique itself is the separately-admitted `tdd` workflow (`AGENTS.md` L219), not this package's job to re-specify |
| BU-P1-011 | `reference/sergeant-upstream/AGENTS.md` L30-31 | PL-4 | J5 (branch-safety constraint) | ABSORBED | `validate-and-ship`'s `10-do-the-work` (`BU-P2-061`: "if the user is on the repository's default branch, a feature branch must be created first") states the identical constraint already |
| BU-P1-012 | `reference/sergeant-upstream/AGENTS.md` L32-33 | PL-4 | J5 (no mode waives validation/review/gate) | ABSORBED | `validate-and-ship` *is* "the single final shipping boundary" (its own `CONTEXT.md` Purpose); this package's own `05-shipping-gate` already delegates to it for exactly this reason |
| BU-P8-058 | `reference/sergeant-upstream/docs/using-sergeant.md` L26 | PL-4 | J4 (only at the boundary the coordinator/Captain has actually approved) | ABSORBED | `validate-and-ship`'s own Trigger line ("Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary") states this exactly |
| BU-P1-013 | `reference/sergeant-upstream/AGENTS.md` L34-35 | PL-4 | J5 (delivery not complete until CI/review/merge conditions hold) | ABSORBED | `validate-and-ship`'s `60-close-out` ("stop driving at `checks-passed`"; drives PR/CI/review to a terminal outcome) |
| BU-P1-014 | `reference/sergeant-upstream/AGENTS.md` L36 | PL-4 | J1 (local record-keeping, no material decision) | ABSORBED | `validate-and-ship`'s `60-close-out` ("any coordinator ownership transfer during the run is durably logged") |

No unit in this package classifies to PL-3 (actor skill/shared method) or
PL-7 (engine gap); none required a new deterministic helper beyond what
`validate-and-ship` and the standard workflow loop already provide.

## Surviving package design

None. Every behavior unit is either already stated elsewhere (`ABSORBED`),
needs a small textual fold into an already-admitted surface (`FOLD`), or
was itself only packaging for other units (`RETIRE`). No unit earns a new
Captain skill, actor skill, workflow, or stage of its own. This confirms
— by applying the method to the package's actual current content, not by
assuming the pilot's own hint — the package-specific hypothesis given at
dispatch (REHOME to a Captain skill routing shipping/review to
`validate-and-ship`), with one correction: the surviving content is thin
enough (one routing invariant restatement plus one worktree-reconciliation
check) that it does not justify authoring a *new* `skills/<name>/SKILL.md`
package. It folds directly into the two surfaces that already carry this
class of judgment — `AGENTS.md`'s routing section and
`validate-and-ship`'s directly-invoked entry — rather than adding a third.
Precedent: `skills/sergeant-help/SKILL.md` shows a retired workflow's
surviving content landing inside an already-admitted surface rather than
mechanically re-creating a same-shaped new package (§8.8 of the proposal:
"a source file mapping one-to-one onto a new package is not evidence of
correctness").

The `.sergeant/workflows/direct-implementation/` directory itself does not
survive: deleted on reconciliation, per the same precedent set by
`sergeant-help` and `grilling`'s prior retirements (both fully removed
from `.sergeant/workflows/` once ported).

## Inputs and outputs

Inputs consumed by this adjudication: the package's own eight files under
`.sergeant/workflows/direct-implementation/` (`CONTEXT.md`, `index.md`,
`workflow.toml`, `_config/standing-constraints.md`, five stage
`CONTEXT.md`/`output/README.md` pairs); `AGENTS.md` (current, in full);
`docs/DEVELOPMENT.md` §"Working on sergeant-rs itself" cross-reference;
`.sergeant/workflows/validate-and-ship/CONTEXT.md` and its
`00-check-scope`/`10-do-the-work`/`60-close-out` stage `CONTEXT.md` files
(the delegation target and its directly-invoked entry, which turned out to
already own most of this package's surviving behavior);
`skills/sergeant-help/SKILL.md` (retirement precedent). No `provenance.md`
exists under this package's directory despite both `CONTEXT.md` and
`workflow.toml` referencing one — noted as a gap below, not fabricated.

Output: this record, plus `draft/direct-implementation/` (retirement stub
and destination-fold proposals, not a live edit — `docs/adr/0013` decision
6, promotable-only review).

## Review and promotion policy

Artifact class: package-adjudication record plus draft fold proposals.
Draft location: `docs/gauntlet/runs/icm-r2/direct-implementation/` (this
record) and its `draft/` subdirectory (proposed destination content).
Independent reviewer: the ICM-R2 pilot's own review step (a later,
independently-executed stage per `docs/adr/0013` decision 7 — fresh
execution, explicit inputs, review-only contract, no edit authority over
this record). Acceptance criteria: behavior-unit dispositions all resolve
to a real destination or an explicit RETIRE rationale; the PL-4
contradiction argument holds up against adversarial challenge; no
citation is fabricated. Promotion action: Captain's reconcile-and-publish
pass (§8.12) accepts/rejects/merges/parks each finding, then — only if
accepted — deletes the live `.sergeant/workflows/direct-implementation/`
directory, applies the `draft/` fold proposals to `AGENTS.md` and
`validate-and-ship`, and updates `.sergeant/index.md` and any routing
table in the same change. Failure/remediation: if review finds the PL-4
contradiction argument does not hold (e.g., a future engine capability
makes "dispatch that returns control to the same conversation" coherent),
the package reverts toward `STAND` and this record is superseded, not
silently edited.

## Alternatives considered

- **STAND** (keep as a five-stage published workflow). Rejected: fails its
  own PL-4 discriminator (§5.6 of the proposal) — the trigger names
  exactly the condition under which the current product does not dispatch.
- **REHOME whole package to a new `skills/direct-implementation/SKILL.md`**
  (the pilot's own stated hypothesis). Considered seriously and closest to
  correct, but rejected as this record's final shape: once each behavior
  unit is actually classified, nothing survives that a new package would
  own independently — `validate-and-ship` already owns the substantive
  work (claim, implement, validate, gate, PR, record), and `AGENTS.md`
  already owns the routing judgment. A new skill whose entire body would
  be "see `AGENTS.md`'s routing section, then run `validate-and-ship`
  directly" is a wrapper, not a package that "faithfully owns" behavior
  (PL Ladder §5.1) — HARVEST into the two existing surfaces is the
  lower-authority, smaller-surface representation the ladder asks for
  first.
- **SPLIT into a Captain-skill stub plus leaving the five stages as an
  actor-only workflow reachable only via that skill's internal call.**
  Rejected: no such invocation mechanism exists yet (`docs/icm/
  convention.md` §4 — context composition today, not true nested-workflow
  invocation); inventing one here would be an unjustified PL-7 engine-gap
  claim disguised as a package decision (§5.9's evidence bar), when the
  actual behavior already has a home without it.

## Final disposition
HARVEST

## Validation evidence

- Grepped `AGENTS.md`, `docs/DEVELOPMENT.md` for `direct.mode`,
  `sgt-context`, `td context`, `TDD` — confirmed the package's cited
  upstream mechanisms (`sgt-context`, `td`) do not exist in the current
  product's live documentation; confirmed the routing trigger and
  "no lighter path" claims already exist in `AGENTS.md`, in different
  words but the same substance, before proposing ABSORBED/FOLD.
- Read `.sergeant/workflows/validate-and-ship/CONTEXT.md` and its
  `00-check-scope`, `10-do-the-work`, `40-drive-gates`,
  `50-reconcile-custody`, `60-close-out` stage contracts in full to verify
  the overlap claims in the table above rather than assuming them from
  the workflow's name.
- Read `skills/sergeant-help/SKILL.md` as a real precedent for "retired
  workflow, content folded into an already-admitted surface, directory
  deleted" before proposing the same shape here.
- Applied PL-4's own required-properties list (§5.6) directly to this
  package's Purpose/Trigger text rather than relying on the pilot
  dispatch instruction's hint as the reason; the contradiction is
  independently reproducible from the package's own `CONTEXT.md` alone.
- Gap noted, not fabricated: `CONTEXT.md` and `workflow.toml` both
  reference a `provenance.md` (stage-to-behavior-unit mapping) that does
  not exist under `.sergeant/workflows/direct-implementation/` at the time
  of this adjudication — flagged for the independent reviewer, not
  invented to fill the citation trail.
- Gap noted: every stage `CONTEXT.md` in this package uses a `## Judgment
  required` heading, not the `## Bounded judgment` heading
  `docs/icm/convention.md` §6.1 and `docs/adr/0013` decision 4 require of
  every actor stage (with named J2/J1/J0 subsections). Immaterial to the
  HARVEST verdict (the package doesn't survive to need the fix), but
  worth the independent reviewer's attention as a corpus-wide drift signal
  — other still-`STAND`ing ICM-R2 packages should be checked for the same
  stale heading.
