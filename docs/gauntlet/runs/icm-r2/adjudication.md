# ICM-R2 — pilot adjudication

Nine-package pilot reconciliation (`docs/adr/0013-icm-r0-owner-rulings.md`
decisions 8–9), following ICM-R1's doctrine landing
(`.sergeant/common/contexts/bounded-judgment.md`, `docs/icm/convention.md`
§6, `docs/icm/record-shapes.md` §6, `_config/icm-ladder.md` §6.1a). Every
package ran producer → independent reviewer (18 dispatched `sgt` Works,
self-hosted, plus one redispatch after a turn-ceiling block on the largest
package), then Captain's own reconcile-and-publish pass applied each
confirmed verdict to the live content. Full per-package record:
`docs/gauntlet/runs/icm-r2/<package>/{adjudication-draft,review}.md`.

## Results by package

| Package | Verdict | What changed |
|---|---|---|
| `grilling` | STAND | Added required Bounded-judgment section (none existed). Fixed a broken citation in "Failure behavior" (cited the wrong engine mechanism). |
| `sergeant-help` | STAND | Added required Bounded-judgment section. Incorporated a J5→J3 re-rung correction for the `AGENTS.md` routing hand-off. |
| `research` | STAND | Added Authority envelope + Bounded-judgment section, including the named J0 clause directly fixing backlog item B9 (worktree-isolation escape). Fixed a stale, now-false rung rationale and deduped drift-prone citations. |
| `validate-and-ship` | STAND | Added Authority envelope + real per-stage Bounded-judgment sections across all 7 stages. Fixed a dangling `route-review-findings` delegation and every `provenance.md` citation (file never existed in-package). Corrected an inaccurate pre-push-hook claim. **Added the honest J0 push/pr/ci placeholder (BU-VAS-15)** — a citable, unresolved gap, not an invented policy. |
| `code-review` | STAND, restructured | Merged two sequential stages into one concurrent dispatch, matching the source. Added a diff-capture helper and the 12-item smell baseline that were classified for this package but never materialized. Fixed a stray duplicate citation caught by independent review after the producer's own self-check missed it. |
| `repo-to-icm` | STAND | Added Authority envelope + Bounded-judgment sections (10 actor stages + adapted execute-stage form). Corrected its own `icm-ladder.md` §6.1a citation, introduced by this same branch's ICM-R1 landing. |
| `task-intake-and-route` | **ABSORBED** | Retired. Every behavior already duplicated on an already-published surface. |
| `sergeant-setup` | **SPLIT** | Retired. Folded into `skills/estate-navigation/SKILL.md` (interactive registration, capability-gap filing) and `AGENTS.md` (the `td`/Graphify/Treehouse confirmation guardrail). Closes backlog item B7, with a correction: `load-project` never had the interview/preview logic B7 assumed. |
| `direct-implementation` | **HARVEST** | Retired. Folded two behaviors into `AGENTS.md`'s "When NOT to use `sgt`" that weren't stated there. |

**Catalog: 23 → 20 published workflows.**

## Method notes

- Every producer applied §8's Contract→Inventory→Harvest→Normalize→
  Placement→Authority→Synthesis sequence and classified behavior units
  before synthesizing a package verdict, per record-shapes.md §6 rule 1.
- Three of nine producers reached a verdict different from — and better
  evidenced than — their own dispatch hint: `task-intake-and-route`
  (hint: SPLIT/HARVEST → actual: ABSORBED), `direct-implementation`
  (hint: REHOME → actual: HARVEST), `sergeant-setup`'s B7 finding (hint:
  delegate to `load-project` → actual: `load-project` doesn't have the
  logic to delegate to). None of the nine dispatch hints was trusted
  without independent verification against current content.
- Independent review caught real defects the producer's own self-check
  missed in at least two cases: `code-review`'s stray `BU-P2-002`
  duplication, and `validate-and-ship`'s dead `provenance.md` citations
  present in every stage.
- One process defect surfaced and was not repeated: the first-round
  assumptions critic (ICM-R0) and, separately, one ICM-R2 producer both
  hit friction from the orchestrating session's own working-directory
  discipline (editing the outer checkout instead of the estate clone;
  see GAUNTLET.md's ICM-R0 entry). Backlog item B9 (a Work navigating
  outside its own worktree) did not recur once given an explicit
  surface-boundary instruction, and `research`'s own pilot amendment now
  encodes that instruction permanently in the package itself rather than
  relying on every future dispatcher to restate it by hand.
- One Work (`repo-to-icm` producer, first attempt) blocked on turn-ceiling
  exhaustion after trying to delegate its own large read/classify/write
  task to a nested sub-agent. Redispatched with an explicit
  no-sub-delegation instruction and a longer ceiling (1500s); succeeded
  cleanly on retry.

## What this does not do

Per ICM-R0's own owner ruling (decision 3), `src/workflows/software-
change/` remains explicitly out of scope. No `src/`, API, journal, or
`workflow.toml`-grammar change landed anywhere in this pass, per the
hard runtime freeze (decision 10) — every change in this pilot is
Markdown, `_config`, or `references/`/`scripts/` content.

The remaining 16 published workflows (`cross-repo-work`, `deepen-module`,
`diagnose-bug`, `dispatch`, `implement`, `load-project`, `prototype`,
`recover-stalled-worker`, `resolving-merge-conflicts`, `tdd`, `to-spec`,
`to-tickets`, `triage`, `vet-external-skill`, `wayfinder`, `worker-
mission`) and the built-in `software-change` workflow are unreconciled —
ICM-R3's subject, not this one's, per the pilot-before-full-corpus
sequencing this workstream itself required (decision 9; proposal
Acceptance Contract item 30).
