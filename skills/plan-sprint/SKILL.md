---
name: plan-sprint
description: Run a multi-wave sprint end to end — recon, plan, panel, rulings, waves, finalize — codifying the integration-branch/wave-branch/blind-panel method this estate's completed sprints have actually used. Use when a body of work needs a sprint-shaped plan with an integration branch, wave branches, and a blind panel gate before waves run.
edition: 0.2.0
---

**[proven]** — new authorship, this skill's protocol is VERBATIM-faithful
to three independently-run sprints this estate has actually completed.

*Why a Captain skill and not a workflow:* the method's mechanical
vocabulary — "integration branch," "wave PR," "CI by SHA" — has zero
footprint anywhere in `.sergeant/`; its load-bearing moments are owner
rulings made live, in conversation, before waves run. Categorically
Captain's under P1.

## When to use

A body of work needs a sprint-shaped plan with an integration branch,
wave branches, and a blind panel gate before waves run.

## The interactive protocol

**The protocol this skill codifies, VERBATIM-faithful across all three
independently-run sprints this estate has actually completed** (quoted,
not paraphrased):

> Foundation (`sergeant-rs-workspace/knowledge/evidence/resources/
> foundation-series/sprint-plan-2026-08-21.md`): "integration branch `integration/
> foundation`, draft head PR carrying this plan, wave branches
> `foundation/w<N>-<slug>` in `/var/tmp/foundation-impl/` worktrees, per
> wave: recon → spec → implement (TDD, DataDir guards, R-S0-12 full loop)
> → 4-axis blind panel (spec-fidelity / invariants / simplicity /
> test-honesty) + per-axis refuters defaulting to refuted → fixer on
> confirmed findings only → wave PR → merge to integration. Sonnet
> subagents by default, opus where earned (named per wave below), Fable
> never in subagents. Rung citations (R/J) in every wave PR body."

> Codex (`sergeant-rs-workspace/knowledge/evidence/resources/h-series/
> sprint-plan-codex-2026-08-21.md`): "integration branch
> `integration/codex`, draft head PR carrying this plan, wave branches
> `codex/w<N>-<slug>` in `/var/tmp/codex-impl/` worktrees (warm base
> build first), per wave: spec → implement (TDD, DataDir guards,
> R-S0-12) → 4-axis blind panel + refuters (default refuted) → fixer on
> confirmed only → wave PR → CI by SHA → merge to integration. Sonnet
> subagents, opus where earned (named below), Fable never below the
> captain. Rung/ruling citations in every PR body."

> CI/CD hardening (main-only at the time this skill was authored —
> `sergeant-rs-workspace/knowledge/evidence/resources/cicd-hardening-series/
> sprint-plan-2026-08-20.md`): "integration branch
> `integration/cicd-hardening`, draft head PR carrying this plan, wave
> branches `cicd/w<N>-<slug>` in `/var/tmp/cicd-impl/` worktrees, per
> wave: recon → spec → implement → 4-axis blind panel (spec-fidelity /
> invariants / simplicity / test-honesty) + per-axis refuters defaulting
> to refuted → fixer on confirmed findings only → wave PR → merge to
> integration. Sonnet subagents by default, opus where earned, Fable
> never in subagents. Only the owner merges main."

Two real points of variance across these three independently-commissioned
sprints, not drift, worth noting rather than silently flattening into one
phrasing: "CI by SHA" appears only in the codex plan; "recon" as an
explicit first step appears in the foundation and CI/CD plans, while the
codex plan's own recon is `h0-adjudication-evidence-2026-08-21.md` —
functionally the same step under a different name.

**The six named phases this skill's procedure states**, synthesized from
the three quotes above plus the artifacts each sprint actually produced:

1. **Recon** — fan out evidence-gathering seats (`recon-results.json` /
   `h0-adjudication-evidence-*.md` / `w<N>-recon.md` per wave — naming
   varies, the step doesn't) against live-verified code/registry truth,
   never doc claims taken on faith.
2. **Plan** — author a single sprint-plan document from recon, citing
   spec sources in J3 authority order, stating the protocol paragraph
   itself and per-wave scope.
3. **Panel** — the plan itself goes through a 4-axis blind panel
   (spec-fidelity / invariants / simplicity / test-honesty) plus
   per-axis refuters defaulting to refuted, exactly as this estate's
   `panel`/`refute` stage vocabulary specifies — `plan-panel-results.json`
   is this step's artifact in all three sprints.
4. **Rulings** — confirmed amendments fold into the same plan document as
   numbered, dated entries with an explicit supersedes clause, never a
   silent rewrite; genuinely open items go to the owner as a live
   grilling or a kickoff-rulings exchange **before** waves run.
5. **Waves** — the per-wave loop quoted above, run strictly in the
   plan's declared order, each wave rebasing on integration head before
   its PR.
6. **Finalize** — version-bump proposal, CHANGELOG, a ratify-at-review
   list carried to the owner at the head PR, and a retro in the fixed
   four-section shape (confirmed identical, verbatim heading text, in
   both retros read for this skill's authoring): `## Timeline` /
   `## What the gauntlet caught that would have shipped` /
   `## Method notes` / `## Owner follow-ups (...)`.

**The explicit limitation this skill's own text states, always:** its
recon and panel seats are **harness sub-agents spawned inside this
session**, not Sergeant Works — true isolated child Works are blocked on
this engine's own filed gaps (no child-workflow dispatch; no
worker-submit-from-surface), which this skill does not pretend around.
State this limitation plainly rather than implying isolation guarantees
the shipped `panel`/`refute` stages also don't actually have.

**Model-assignment policy — referenced, not restated.** Every seat this
method spawns (recon, plan-panel, per-wave implement/panel/refute/fix
seats) is governed by `.sergeant/common/contexts/model-assignment.md`.
This skill names that file as the governing policy; it does not re-copy
"sonnet default, opus where earned, Fable never below the captain" into
its own body — a fifteenth verbatim copy of that policy is the exact
copy-pasted-into-every-plan defect this kernel exists to stop repeating.

## Bounded judgment

### This skill may decide
- Wave ordering and scope-per-wave, within a plan the owner has already
  ratified.
- Which sub-agent seat count a given recon/panel step needs.

### This skill must ask the user
- Every ratify-at-review item, live, at the point the plan names it
  deferred — never decided on the owner's behalf.
- The owner merges main, always — every one of the three plans read for
  this skill states this.

### This skill must not do
- Dispatch waves as Sergeant Works — state the sub-agent limitation
  instead, per above.
- Decide a ratify-at-review item without the owner.
- Write its own protocol into `AGENTS.md` — the method's vocabulary has
  zero engine footprint and stays out of the always-on file for exactly
  that reason.
- Re-copy the model-assignment policy text inline — name
  `.sergeant/common/contexts/model-assignment.md` instead.
- Run this skill's own planning/rulings turns via `sgt run` or any
  durable Work dispatch — the load-bearing moments are owner rulings made
  live, in conversation.
- Present an unconfirmed, harness-degraded best guess as a ratified
  ruling; say so plainly instead.

### Durable handoff
The sprint-plan document itself (with panel amendments folded in as
dated, numbered entries), the retro, and the ratify-at-review list on the
head PR — all workspace/product-repo Markdown artifacts, the same shape
every sprint plan this estate has run already produces.

## Failure behavior

If this invocation has no live human who will send the next message, this
skill cannot put ratify-at-review items to the owner live. Say so plainly
and leave those items recorded as open in the plan document rather than
deciding them unilaterally.
