# ICM-R3 — full reconciliation adjudication

Sixteen-package reconciliation — every workflow package left unreconciled
after ICM-R2's nine-package pilot (`docs/adr/0013-icm-r0-owner-rulings.md`
decisions 8–9), closing out the library-reconciliation phase of the
Placement/Bounded-Judgment doctrine landing. Every package ran producer →
independent reviewer (32 dispatched `sgt` Works across three waves, plus
one final single-Work dispatch that mechanically applied six
already-adjudicated verdicts to live files), then Captain's own
reconcile-and-publish pass applied each confirmed verdict to the live
content. Full per-package record: `docs/gauntlet/runs/icm-r3/<package>/
{adjudication-draft,review}.md`.

## Results by package

| Package | Verdict | What changed |
|---|---|---|
| `tdd` | **REHOME** | Retired. Folded into `.sergeant/common/contexts/tdd.md` and a new `test-quality.md` as shared contexts (`@@tdd`/`@@test-quality`), cited from `implement` and `worker-mission`. Named only two stages and was a technique (test-driven development), not a workflow — the producer's own REHOME finding was disputed by its reviewer citing a missed PL-7 alternative; resolved by direct owner ruling, not left unresolved. |
| `implement` | STAND | Authority envelope; corrected both stages' delegations (`tdd`→`@@tdd`; `code-review`→dispatched as its own Work, not context composition); real Bounded-judgment sections. Filed (not fixed) an engine-gap claim for nested-workflow invocation, cross-linked to the pre-existing G6 finding rather than duplicated. |
| `worker-mission` | STAND | Authority envelope; all 4 stages' Bounded-judgment, including a new J0 clause for triage candidates straddling more than one of five categories; `20-implement`'s delegation corrected the same way as `implement`'s; `provenance.md` self-reference fixed. |
| `to-spec` | **REHOME** | Retired. Folded into `skills/to-spec/SKILL.md`; `AGENTS.md` routing row added. Its defining behavior — synthesize from "the current conversation," never by interview — names a live-dialogue dependency a dispatched Work cannot receive, the same PL-4 failure `direct-implementation` hit at ICM-R2. Both reviewer-required revisions applied: seam-content dedup note vs `@@tdd`, added missing "Failure behavior" section. The `ready-for-agent`/triage-label J0 conflict this skill surfaces against `triage` is preserved as a live, unresolved gap, not silently resolved. |
| `diagnose-bug` | STAND | 6 stages' Bounded-judgment; dangling `/improve-codebase-architecture` reference fixed; `provenance.md` fixed; a J5 citation corrected to its real source line. |
| `prototype` | STAND | 6 stages including two mutually-exclusive branches; new `references/shared-rules.md` created (4 behavior units extracted at N1 but never materialized — a reviewer correction from an initial "harvest gap" misclassification). Item 4, "skip the polish," remains a genuine, explicitly recorded unresolved gap. Capture-trigger language made uniform across branches. |
| `deepen-module` | STAND | 3 stages; restored 3 missing behavior units (BU-P4-018/019/025); corrected BU-P4-023's J-rung from J1 to J5 per reviewer dispute; `provenance.md` fixed incidentally. |
| `dispatch` | STAND | 6 stages; fixed two dangling references to never-built packages (`drain-fleet`, `respond-to-worker` — both name the open engine gap G4, not live delegations); corrected BU-DISP-04's J-rung from J2 to J5 per reviewer dispute; `provenance.md` fixed. |
| `load-project` | **ABSORBED** | Retired. Every behavior already duplicated on `estate-navigation`, matching `task-intake-and-route`'s ICM-R2 precedent exactly. |
| `triage` | STAND, restructured | Renumbered `20-verify`/`30-recommend` → `20-recommend`/`30-verify` (BU-TRI-04): verify's own trigger text and the upstream source's own line order both place verification after recommendation, not before. Authority envelope; Bounded-judgment across all 5 stages, including `20-recommend`'s J0 gate (no state-changing action before maintainer direction) and `50-apply-outcome`'s J5 KB write/no-write rules (BU-P3-069 through BU-P3-096). `provenance.md` pointer fixed to `docs/gauntlet/promoted-provenance/triage.md`. |
| `wayfinder` | STAND | 4 stages amended with Authority envelope + Bounded-judgment. |
| `resolving-merge-conflicts` | STAND | 2 stages amended with Authority envelope + Bounded-judgment. |
| `vet-external-skill` | STAND | 7 stages amended with Authority envelope + Bounded-judgment. |
| `recover-stalled-worker` | STAND | 3 stages amended with Authority envelope + Bounded-judgment. |
| `cross-repo-work` | STAND | 5 stages amended with Authority envelope + Bounded-judgment. |
| `to-tickets` | STAND | 4 stages amended with Authority envelope + Bounded-judgment. `00-load-project-context`'s delegation citation retargeted from the retired `load-project` to `estate-navigation`'s "Resolving estate context" section. |

**Catalog: 20 → 17 published workflows.**

## Method notes

- Every producer applied §8's Contract→Inventory→Harvest→Normalize→
  Placement→Authority→Synthesis sequence and classified behavior units
  before synthesizing a package verdict, per `record-shapes.md` §6 rule 1.
- Worktree-exclusivity friction discovered and fixed mid-run: parallel
  wave-1 dispatch initially had all producers instructed to check out a
  shared branch by name, which is mutually exclusive across every
  worktree sharing the estate clone's object store (including Captain's
  own checkout). Fixed by switching dispatch instructions to
  `git fetch origin <branch> && git checkout --detach FETCH_HEAD`, and by
  moving Captain's own consolidation work to a separate outer checkout
  with its own `.git`.
- One disputed verdict (`tdd`'s REHOME) was resolved by direct owner
  ruling rather than left as an unresolved dispute in the record — see
  the corrected `docs/gauntlet/runs/icm-r3/tdd/adjudication-draft.md` for
  the reasoning: only-two-stages-is-a-technique, PL-7 alternative
  explicitly considered and rejected.
- The final six packages (`wayfinder`, `resolving-merge-conflicts`,
  `vet-external-skill`, `recover-stalled-worker`, `cross-repo-work`,
  `to-tickets`) were mechanical apply-the-already-decided-verdict work —
  each already had a producer draft and independent reviewer verdict on
  record. Per the owner's own observation mid-session, this class of work
  was dispatched as a single `sgt run` Work rather than continued as
  manual Captain-direct file edits, closing each package as its own
  separate, independently verifiable commit.
- A `git add`/commit silently dropping staged files despite the commit
  message claiming them recurred once this session (`to-spec`'s
  `skills/to-spec/SKILL.md` and `AGENTS.md` routing row) — the same
  failure mode caught once already at ICM-R2. Caught the same way: running
  `git status` before moving on, not assumed clean. Fixed with an explicit
  follow-up commit.
