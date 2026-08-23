# Sprint plan — Distro content rebuild: Captain's skills, Sergeant's workflows (2026-08-22)

Owner-commissioned standard autonomous sprint. Spec authority (J3, in
order): the kickoff rulings + owner testimony in
`design-proposal-2026-08-22.md` (this dir — revision 2, panel-derived,
external-review-merged, scope-ruled); the design proposal's sections
themselves for every mechanism detail; the external review
(`external-review-2026-08-22.md`) as merged lead material, never
independently authoritative. Aspiration on record: after this sprint the
estate swaps to using sergeant for everything.

**Protocol** (unchanged): integration branch `integration/distro-content`,
draft head PR carrying this plan, wave branches `distro/w<N>-<slug>` in
`/var/tmp/distro-impl/` worktrees (warm base build), per wave: spec →
implement (R-S0-12 where code, content-fidelity where content) → 4-axis
blind panel + refuters (default refuted) → fixer on confirmed only → wave
PR → CI by SHA → merge to integration. Sonnet workers, opus where earned
(named per wave), Fable never below the captain. Rung/ruling citations in
every PR body. Owner merges main.

**Scope (owner-ruled):** `skills/`, the shipped workflow packages,
`AGENTS.md` — plus exactly one granted engine exception (W1) and the
content-in-spirit riders (src/workflows/ sources, `.sergeant/index.md`,
package `version` fields, the m-suite tests pinning changed behavior).

## Waves

### W1 — Retire the embedded default; bare execution (the granted engine exception)
- Remove `src/workflows/software-change/` and the `DEFAULT_WORKFLOW`
  fallback (src/domain/workflow.rs ~64, 600-621). Unspecified
  `--workflow` binds the **bare execution scaffold**: minimal
  intent→actor→close (spec decides: an embedded minimal scaffold vs a
  truly optional binding — whichever is smaller and honest; no pretend
  review stage, no fake structure).
- The Work view/journal record what was bound ("bare" is a first-class,
  visible answer — never a silent fallback; the routing journal already
  records route_source, extend the same honesty to structure).
- All m-suite tests referencing software-change updated; a test pins that
  bare execution reaches terminal with honest integrity reporting; the
  61-Work precedent replays cleanly (existing journals must still fold —
  the retired workflow's journaled references are history, not errors).
- Panel: invariants seat owns replay compatibility (journaled
  software-change bindings from old Works must not break projection).

### W2 — Sergeant's workflow core (7 packages + stages/policies content)
Opus spec (earned: the panel-stage design inside implement-change is the
sprint's correctness core).
- **implement-change**: the flagship — orient → baseline → implement
  (test-first policy) → targeted validation → **20-panel (4 axes, in-stage
  sub-agent fan-out per code-review's precedent) → 25-refute (default
  refuted) → 30-fix-confirmed → 35-re-verify (over fix commits)** →
  evidence/close. The panel/refute contract text lives in shared contexts
  so review-change shares it by convention (F.3's residual, per
  recommendation).
- **fix-defect** (diagnose-bug reshaped 6→8), **investigate** (1→6,
  absorbing wayfinder minus its R-NS-6-violating stage 00),
  **review-change** (multi-axis, verified-findings-not-fixes, typed
  finding set per the product-rules adoption), **remediate-findings**
  (every finding accounted: accept/reject/supersede + disposition matrix),
  **author-document** (record-decisions becomes its profile-section per
  F.6, flagged), **validate-and-ship** (reshaped; delivery-state as
  content policy, not engine).
- **Retirements/absorptions per the disposition table**: dispatch (cut),
  cross-repo-work (dissolved: topology belongs to the intent),
  prototype, vet-external-skill, triage, to-tickets (roadmap),
  resolving-merge-conflicts (shared stage), deepen-module (roadmap
  technique), repo-to-icm (workspace-local move), recover-stalled-worker
  (engine lead), validate-intent (folds to W3's define-acceptance),
  record-decisions (into author-document). The tdd dangling references
  die with their hosts; grep-proof no dangler survives.
- **Stages + policies content**: the nine reusable stage contexts;
  `.sergeant/common/contexts/` gains the policy texts (test-first
  consolidated, independent-review, evidence-requirements,
  model-assignment — un-copy-pasted from sprint plans at last);
  `.sergeant/index.md` rebuilt; per-package `version` bumps per ADR 0016.
- Panel axes: content-fidelity (vs the design proposal §C-§G),
  R-NS-6/doctrine, simplicity (no prompt blobs; stages genuinely
  reusable), robustness (the owner's bar: multi-stage, crash-legible,
  no fragile one-stagers).

### W3 — Captain's kernel + AGENTS.md (doctrine-truth panel axis)
- **14 skills**: orient, brainstorm, clarify-intent, scope-intent,
  define-acceptance (absorbing validate-intent), decide (carrying J0's
  one-question discipline — adjudicate folded), decompose,
  select-workflow, review-outcome, retrospective, grilling (+
  grill-with-docs seamed at the brief per F.6), plan-sprint (the
  three-sprint-proven method, with the explicit caveat its seats are
  harness sub-agents until the engine leads land), estate-navigation,
  sergeant-help. Each with Bounded-judgment + Durable-handoff sections
  per the ADR 0013/0014 convention; each states what it must NOT do.
- **AGENTS.md**: the F.10 dispatch-time discipline at
  stable-operating-invariant rung ("omitting --workflow binds bare
  execution; that is a selection and must be cited as one"); the
  routing-table cell corrected (Captain is the selector — the documentary
  root of sixty unrecorded choices); catalog-awareness routing to
  select-workflow; the policies' AGENTS.md homes; "Captain owns
  ambiguity, Sergeant owns completion" as the boundary statement beside
  R-NS-6.
- Panel: doctrine-truth axis holds every AGENTS.md sentence against the
  code and rulings (the P6 no-catalog-copy rule; nothing overclaims).

### W4 — Finalize + proof
- Version bump proposal: **0.3.0** (removing the embedded default is
  user-visible behavior; ratify-at-review), CHANGELOG, README touch-ups
  where the catalog is named, docs-consistency green.
- **Proof, dogfooding the aspiration**: re-run `sgt init` on the
  workspace estate (idempotent update to the new content); dispatch one
  real Work through **implement-change** (luna/terra per owner word at
  the time) exercising the in-stage panel; dispatch one **bare-default**
  Work proving the honest null case; both journal-verified. Retro;
  cleanup; head PR un-drafted with the ratify list (F.3 residual, F.4
  per-item, F.5 fold cost, F.6 profile promissory note, F.7 freeze-risk,
  F.8 edition mechanics, 0.3.0).

## Wave ordering & conflict control
W1 → W2 → W3 → W4 strictly serial (W2's packages assume W1's bare-default
semantics; W3's AGENTS.md names W2's catalog; W4 proves the lot). Each
wave rebases on integration head before its PR.

## Risks
- **The panel-in-a-stage is the novel machinery** (in-stage sub-agent
  fan-out at 4 axes + refuters, on code-review's 2-seat precedent): W2's
  spec must state the isolation limits honestly (weaker than the sprints'
  process isolation) and the proof Work must exercise it live before the
  head PR claims it works.
- **Replay compatibility** (W1): 60 journaled Works reference the retired
  workflow; projection must fold them forever.
- **Content fidelity at scale**: ~21 packages touched; the per-wave
  panels carry content-fidelity axes against the design proposal, and
  grep-proofs (danglers, stale names) are wave gates.
- tmpfs: builds in `/var/tmp/distro-impl/` only.

## Panel amendments (binding — 2026-08-22 plan panel, 2 confirmed / 6 refuted)

**A1 (MAJOR, content-integrity) — the engine exception's true footprint.**
Removing the embedded default is NOT two files: `WorkflowDefinition::embedded()`
is called from src/api.rs:2321 (`workflow_catalog_entries`, the documented
GET /v1/workflows fallback contract, T2-39/T2-40) — a compile break if
untouched — and src/cli.rs carries the `--workflow` help text plus the
doctor workflow-count check whose stated rationale assumes the embedded
fallback exists; src/domain/distro.rs's module doc cites it as precedent.
W1's spec MUST grep src/ for DEFAULT_WORKFLOW / SOURCE_EMBEDDED /
EMBEDDED_WORKFLOW_TOML / WorkflowDefinition::embedded and enumerate every
hit as in-scope. The widening of the granted exception (api.rs + cli.rs +
distro.rs prose) is FLAGGED TO THE OWNER before W1 lands — never silently
absorbed. W1's spec also decides, deliberately, what GET /v1/workflows
answers post-retirement (named catalog only, with bare execution
documented — exact shape is the spec's call, ratify-at-review).

**A2 (MAJOR, doctrine) — W3's AGENTS.md edit target corrected.** Ruling
(b)'s two edits (dispatch-time discipline; the selector cell) target the
`## Trigger → skill/workflow routing table` (AGENTS.md:65-80), NOT
`### ROUTING — dispatch vs. in-session` (which contains no
workflow-selection language). The two routing concerns stay unconflated.
