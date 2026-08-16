# ICM-R0 fidelity critic — axis 1

Artifact under review: `reference/proposal-icm-r-procedure-authority.md`
(full text, §1–§20 plus Source-to-Decision Map and Owner Decisions).
Contract: `docs/gauntlet/contracts/ICM-R0.md`.

## Method

Read the contract and the full proposal text first, blind to any other
critic's output (nothing under `docs/gauntlet/runs/icm-r0/` was read before
writing this report). Checked every §3 finding (ICMR-F1–F7) and the §12
package hypotheses against the current working tree on
`chore/backlog-grooming-2026-08-16`, not against the proposal's own
`3a46b87c` pin, per the contract's explicit instruction. Verified via direct
file reads and `grep`/`git diff` against:

- `src/runtime/engine.rs`, `src/backend/claude.rs`, `src/domain/workflow.rs`
- `docs/icm/convention.md`, `docs/icm/record-shapes.md`,
  `docs/icm/promotion-spec-2026-08-11.md`,
  `docs/icm/re-homing-record-2026-08-12.md`
- `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md`
- `.sergeant/index.md`, `.sergeant/workflows/*`, `skills/*`
- `.sergeant/workflows/validate-and-ship/40-drive-gates/CONTEXT.md`
- `git diff 3a46b87c17d249655708ed5ac32f6704738776cf..ad20ec7` scoped to the
  above files, to see whether the T-series sprint changed anything the
  proposal's §3 findings depend on.

Scope is fidelity only: does the proposal accurately represent what the
repository, as it stands now, actually contains and does. Invariants,
assumptions-as-truth, and enactability are other critics' axes.

## Findings

### F1 — severity: info — §3.1 (ICMR-F1, stage execution boundary)

**Claim.** "A workflow is resolved and pinned before execution. At stage
entry, the engine selects the current stage's pinned executor, creates a new
execution identity... reserves the native identity, and launches outside the
core lock... a question becomes NeedsInput and resumes through the same
session when `sgt respond` supplies the answer" — and that no runtime
rewrite is needed to make one folder equal one fresh stage execution.

**What I checked.** `src/runtime/engine.rs` (`reserve_stage`,
`stop_execution`, the `BackendSignal::NeedsInput` / `StageCompleted` arms)
and `src/backend/claude.rs` (session-id minting via `--session-id` on first
turn, `--resume <session_id>` on subsequent turns, the module doc's own
description of this contract).

**What I found.** The engine and Claude adapter behave exactly as
described: fresh execution identity per stage attempt, `--resume` reuse only
within the same still-open stage, `stop_execution` called on both
`NeedsInput` and `StageCompleted` paths as the code's own comments describe.
`git diff` against the pin shows zero changes to either file in the
T-series sprint.

**Verdict.** Survives. Clean pass — no drift since the pin, and the
description matches current `main`.

### F2 — severity: info — §3.2 (ICMR-F2, ICM filesystem vs. engine grammar)

**Claim.** The four-layer convention (orientation, stage contract, stable
reference, per-run artifact) exists, downstream stages consume upstream
outputs via named Inputs-table entries, and only `workflow.toml` and each
stage's `CONTEXT.md` are engine-interpreted — the rest is ordinary
Git content the actor navigates.

**What I checked.** `docs/icm/convention.md`'s layer table and its six
numbered layer rules; `git diff` of `docs/icm/convention.md` between the
pin and current `main`.

**What I found.** The four-layer split, the Inputs-table requirement, and
the "Layer 1 is not a super-stage" rule are all present verbatim in
`convention.md`. Zero diff between the pin and current `main` for this
file.

**Verdict.** Survives. Clean pass.

### F3 — severity: info — §3.3 (quoted text from icm-ladder.md §6.2)

**Claim.** The proposal presents, as a blockquote, "§6.2 currently asks:
> Is it a reusable procedural outcome with a recognizable trigger, bounded
outcome, and completion condition?"

**What I checked.** `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md`
§6.2 verbatim text.

**What I found.** The actual file splits this into a heading ("## 6.2 — Is
it a reusable procedural outcome?") and a separate body sentence ("Does it
have a recognizable trigger, a bounded outcome, and a completion condition
that could be invoked independently?"). The proposal's blockquote merges
these two sentences into one and presents the result as a direct quotation.
The substance (trigger, bounded outcome, completion condition, and the
"necessary but not sufficient" argument built on it) is preserved
faithfully — this is a compression, not a misrepresentation of what §6.2
tests for — but formatting it as an exact `>` quote when it is a paraphrase
is a minor citation-fidelity slip, notable mainly because citeability is
this proposal's own central discipline.

**Verdict.** Survives with a minor note. Does not undermine §3.3's
argument (that §6.2 lacks a driver/admission discriminator), which holds
independently of the exact wording.

### F4 — severity: info — §3.4 (ICMR-F4, validate-and-ship bounded judgment)

**Claim.** `validate-and-ship/40-drive-gates` already distinguishes
auto-fix (actor may authorize), no-op (no decision needed), and ask-user
(must go to the user, never resolved autonomously), including a recorded
disputed standing-consent exception.

**What I checked.**
`.sergeant/workflows/validate-and-ship/40-drive-gates/CONTEXT.md` in full.

**What I found.** Matches closely, down to near-verbatim language: "action
classifying it as `auto-fix` (mechanical/low-risk, actor may authorize on
their own judgment), `no-op` (informational, nothing to do), or `ask-user`
(challenges the user's deliberate intent or touches product behavior — a
decision only the user can make)" and the `--yes` standing-consent carve-out
("the sole exception to the ask-user escalation rule"). `git diff` shows
this file is unchanged between the pin and current `main`.

**Verdict.** Survives. Clean pass, one of the strongest-cited findings in
the proposal.

### F5 — severity: info — §3.5 (ICMR-F5, structural vs. semantic validation)

**Claim.** The promotion spec's unscripted fake-backend run completed
`sergeant-setup/30-project-interview` without exercising a single
`needs_input` transition, despite that stage depending on repeated human
answers — proving structural walk-through is not semantic validation.

**What I checked.** `docs/icm/promotion-spec-2026-08-11.md`, specifically
its G5 engine-gap discussion and the `sergeant-setup`/`30-project-interview`
references.

**What I found.** Confirmed near-verbatim: "`sergeant-setup`'s
`30-project-interview` stage's real, adjudicated content depends on a
multi-round `needs_input` loop this unscripted run never exercised," listed
under engine-gap **G5** alongside `grilling`.

**Verdict.** Survives. Clean pass.

### F6 — severity: info — §3.6 (ICMR-F6, prior re-homing round was package-level)

**Claim.** The 2026-08-12 re-homing round retired absorbed packages, moved
conversational packages to skills, and preserved provenance, but assigned
verdicts at the package level rather than the behavior-unit level.

**What I checked.** `docs/icm/re-homing-record-2026-08-12.md` and
`.sergeant/index.md`'s own retrospective note ("12 retired — 9 CLI-SURFACE,
1 OPERATOR-SKILL, and the 2 R-NS-6-dissolved `grilling`/`grill-with-docs`").

**What I found.** `.sergeant/index.md` independently corroborates the
package counts and disposition categories the proposal describes (35 → 23
packages, with grilling/grill-with-docs specifically named as the
R-NS-6-dissolved pair). No discrepancy found.

**Verdict.** Survives. Clean pass.

### F7 — severity: warning — §3.7 (ICMR-F7, "no code prerequisite")

**Claim.** The engine already supports a listed set of capabilities
(ordered pinned stage contexts, fresh execution per attempt, actor/execute
stage kinds, needs_input/respond, retry, shared worktree artifacts, journal
evidence, content-addressed transcripts, draft/admitted publication
boundary) and a placement+authority ladder requires zero new Rust: no new
Work states, event kinds, API fields, workflow.toml fields, TUI controls,
backend capability, artifact database, or scheduler.

**What I checked.** `git diff 3a46b87c17d249655708ed5ac32f6704776cf..ad20ec7`
scoped to `src/domain/workflow.rs`, `src/runtime/engine.rs`,
`src/backend/claude.rs` — i.e., whether anything the T-series sprint landed
after the proposal's pin is a new engine capability the proposal's "already
supports" list should have picked up, or a new API surface that undercuts
the "no new API fields" prerequisite claim.

**What I found.** `src/domain/workflow.rs` gained +415 lines since the pin:
a new `catalog()` function, `CatalogEntry`/`WorkflowIndexFrontMatter`
types, and a new `ROOT_CATALOG_FILE`/`INDEX_FILE` constants pair, backing a
new `GET /v1/workflows` read-only catalog API route (T-series T2-39/T2-40).
This is a genuinely new API route added to `main` after the proposal's
audit pin. It does not contradict ICMR-F7's narrow claim (a placement/
authority ladder needs none of the listed capabilities to exist first) —
the new route is unrelated to stage execution, `needs_input`, or retry, and
the proposal never claims the engine is *frozen*, only that this specific
campaign has no code prerequisite. But it is exactly the kind of
"engine capability that postdates the pin" the contract asks this axis to
watch for, and the proposal's own list in §3.7 (written to be
exhaustive-sounding: "The current engine already supports: ...") is
silently stale by one item as of current `main`. This does not change
ICMR-F7's verdict, but the finding is worth naming since the proposal
explicitly claims its own audit was current as of a pin that a full sprint
has since passed.

**Verdict.** Survives, with a named staleness note. The core claim ("zero
Rust changes needed for the ladder work") is unaffected because the new
route isn't among the things the ladder work depends on — but the
"engine already supports" enumeration is not a complete list of what `main`
now contains, and a future revision of this proposal (or ICM-R1) should
re-audit §3.7 against a fresh pin rather than reuse this list verbatim.

### F8 — severity: info — §12 package hypotheses (six cited packages)

**Claim.** §12 describes current contents of `task-intake-and-route`,
`direct-implementation`, `sergeant-setup`, `dispatch`, `worker-mission`, and
`load-project` as a basis for SPLIT/REHOME/STAND hypotheses.

**What I checked.** Directory listing and `CONTEXT.md` for all six packages
under `.sergeant/workflows/`.

**What I found.** All six packages exist with the stage lists and
subject matter the proposal describes: `task-intake-and-route`'s
`03-choose-mode`/`05-confirm-decisions` stages matching the "shape intent,
decide direct versus durable... resolve user decisions" description;
`direct-implementation`'s stages matching "implementation in the current
session... under the same delivery contract as a dispatched worker";
`sergeant-setup` still split across `05-file-capability-gaps` and
`30-project-interview` as described. No package was renamed, retired, or
restructured by the T-series sprint (confirmed via
`git diff --stat 3a46b87c..ad20ec7` over `.sergeant/index.md`, `skills/`,
and the ICM doc set — zero changes).

**Verdict.** Survives. Clean pass.

### F9 — severity: info — Executive Summary point 4 / §10.4 (23 workflows, 4 skills)

**Claim.** "reconcile all 23 published workflows and four current operator
skills."

**What I checked.** `ls .sergeant/workflows/`, `ls skills/`,
`.sergeant/index.md`'s catalog table.

**What I found.** Exactly 23 workflow directories and 4 skill directories
(`estate-navigation`, `grill-with-docs`, `grilling`, `sergeant-help`),
matching `.sergeant/index.md`'s own count and its "23 packages (down from
35...)" retrospective note.

**Verdict.** Survives. Clean pass.

## Summary

Eight of nine checked areas (F1, F2, F4, F5, F6, F8, F9, and F3's
underlying argument) survive fidelity review with no correction needed —
every specific file, line-level behavior, and package-content claim I
checked against current `main` matched. Two notes, neither severity-error:

- F3: a quoted sentence in §3.3 is a paraphrase presented as an exact
  quotation of `icm-ladder.md` §6.2 — cosmetic, doesn't change the argument.
- F7: §3.7's "already supports" capability list is silently one item stale
  relative to current `main` (a new `GET /v1/workflows` catalog route
  landed in the T-series sprint after the proposal's pin) — doesn't change
  the "no code prerequisite" conclusion, but the audit-pin drift the
  contract's Unknowns section asked this gauntlet to specifically test for
  is real, even if narrow in this case.

No finding here invalidates a section's premise or rises to `error`
severity. On fidelity alone, this proposal's factual claims about the
current repository hold up well against current `main`, a full sprint past
its own stated pin.
