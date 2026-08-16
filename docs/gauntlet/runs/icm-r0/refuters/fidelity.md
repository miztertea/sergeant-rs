# ICM-R0 fidelity refuter — axis 1

Artifact under review: `reference/proposal-icm-r-procedure-authority.md`.
Contract: `docs/gauntlet/contracts/ICM-R0.md`. Critic report under refutation:
`docs/gauntlet/runs/icm-r0/critics/fidelity.md`.

## Method

Read the contract, the full proposal text, and the critic report. For every
finding in the critic report, re-derived the underlying claim independently
against the repository at `HEAD` (`5de7041`, which matches
`chore/backlog-grooming-2026-08-16`'s tip) using direct file reads, `grep`,
and `git diff`/`git log` against the commits the critic and proposal cite —
not by trusting the critic's own quotations. Attempted to find a reading
that breaks each finding before accepting it.

## Findings

### F1 — verdict: CONFIRMED

**Claim under review.** Stage execution boundary (fresh execution identity
per stage, `--session-id`/`--resume` in the Claude adapter, `stop_execution`
on both `NeedsInput` and `StageCompleted`) matches the proposal's
description, unchanged since the pin.

**Independent re-derivation.** `grep -n "fn reserve_stage\|fn stop_execution\|NeedsInput\|StageCompleted" src/runtime/engine.rs` shows `reserve_stage` (engine.rs:2848), `stop_execution` (engine.rs:3820), and both `BackendSignal::NeedsInput`/`StageCompleted` arms present. `grep -n "session-id\|--resume\|session_id" src/backend/claude.rs` confirms the module doc explicitly documents `--session-id` on the first turn and `--resume` after, matching the proposal's and critic's description. `git diff 3a46b87c..HEAD -- src/runtime/engine.rs src/backend/claude.rs` is empty — genuinely zero drift since the pin.

**Attempt to refute.** Tried to find a case where `--session-id` is not actually used to mint identity before launch, or where `stop_execution` is not called on both terminal paths — the grep hits are consistent with the critic's description and no contradicting code path was found.

**Verdict.** Confirmed. No basis to downgrade or strike.

### F2 — verdict: CONFIRMED

**Claim under review.** The four-layer ICM convention, Inputs-table
handoff rule, and "Layer 1 is not a super-stage" rule are present verbatim
in `docs/icm/convention.md`, unchanged since the pin.

**Independent re-derivation.** `git diff 3a46b87c..HEAD -- docs/icm/convention.md` produces zero lines of output — the file is byte-identical to the pinned revision. Since the critic's own quoted content is therefore necessarily still current (nothing changed), the only way to refute F2 would be to show the critic misquoted the pinned file itself, which is out of this axis's scope (the contract asks to check against current `main`, and current `main` here is unchanged from the pin for this file).

**Verdict.** Confirmed.

### F3 — verdict: DOWNGRADED

**Claim under review.** The proposal's §3.3 blockquote of `icm-ladder.md`
§6.2 merges a heading and a body sentence into one and presents it as an
exact quotation — a citation-fidelity slip, not a misrepresentation of
substance.

**Independent re-derivation.** `.sergeant/workflows/repo-to-icm/_config/icm-ladder.md` lines 20-23 read:

```
## 6.2 — Is it a reusable procedural outcome?

Does it have a recognizable trigger, a bounded outcome, and a completion
condition that could be invoked independently? (Examples: diagnose a
```

The proposal's §3.3 renders this as a single `>` blockquote: "Is it a
reusable procedural outcome with a recognizable trigger, bounded outcome,
and completion condition?" This is confirmed as a paraphrase (word order
changed, "invoked independently" dropped, heading and body fused) formatted
as a direct quotation.

**Attempt to refute.** The critic calls this "info" severity and says it
"does not undermine §3.3's argument." That downstream-argument judgment is
correct and independently verified — the driver/admission-boundary gap
argument built on top of this paraphrase holds regardless of the exact
wording. However, the critic's own report explicitly frames this proposal's
"central discipline" as citeability (echoing the proposal's own emphasis on
line-level citation and its evidence hierarchy in §2.5), and then still
rates the violation of that exact discipline as "info" — the lowest
severity available, on par with an uncontested clean pass like F1 or F2.
A formatting slip that fabricates a direct quotation in a document whose
own methodology promises "no claim without citation, no citation without
exact text" is a real, if narrow, fidelity defect distinct in kind from an
uncontested match — it deserves to be visible as a genuine (if minor)
finding rather than folded at the same "info" tier as zero-diff passes.

**Verdict.** Downgraded in effect only insofar as the severity label should
be distinguished from true clean passes (F1, F2, F4-F6, F8, F9); the
underlying factual claim (paraphrase-as-quote) and the "does not undermine
the argument" conclusion are both confirmed as stated. Recorded as
downgraded to flag that F3 is not equivalent in kind to the surrounding
"survives, clean pass" findings, not because any part of the critic's claim
is wrong.

### F4 — verdict: CONFIRMED

**Claim under review.** `40-drive-gates/CONTEXT.md` distinguishes
auto-fix/no-op/ask-user, includes near-verbatim language matching the
proposal's §3.4, and records a disputed standing-consent exception, unchanged
since the pin.

**Independent re-derivation.** Read `.sergeant/workflows/validate-and-ship/40-drive-gates/CONTEXT.md` in full (108 lines). The auto-fix/no-op/ask-user classification is present near-verbatim (line 33). The "sole exception to the ask-user escalation rule" language for `--yes` is present verbatim (line 60). Initially suspected the "disputed" framing might only exist in a different file (`docs/icm/promotion-spec-2026-08-11.md`'s Conflict X3), which would make the critic's claim about *this specific file* overstated — but the file's own "## Additional note" section (line 104) explicitly states: "Conflict X3 (synthesis.md §6): whether `--yes` unattended consent may ever be used is contested between an absolute-never reading... and a documented standing-consent exception... this draft follows the absolute-never reading... and preserves the exception as evidence, not as an instruction to follow." The dispute is recorded in this exact file. `git diff 3a46b87c..HEAD` on this file is empty.

**Attempt to refute.** The refutation attempt (that the "disputed" claim was displaced to a different file) failed on direct inspection — the file does contain it.

**Verdict.** Confirmed. This is a strong, fully-verified finding.

### F5 — verdict: CONFIRMED

**Claim under review.** `docs/icm/promotion-spec-2026-08-11.md`'s G5
engine-gap discussion confirms the unscripted fake-backend run completed
`sergeant-setup/30-project-interview` without exercising a `needs_input`
transition.

**Independent re-derivation.** `grep -n "G5\|needs_input\|30-project-interview" docs/icm/promotion-spec-2026-08-11.md` surfaces the exact passage at lines 223-247, including "`sergeant-setup`'s `30-project-interview` stage's real, adjudicated content depends on a multi-round `needs_input` loop this unscripted run never exercised" under engine-gap G5. `git diff 3a46b87c..HEAD` on this file is empty.

**Verdict.** Confirmed.

### F6 — verdict: CONFIRMED

**Claim under review.** `.sergeant/index.md`'s retrospective note
corroborates the 2026-08-12 re-homing round's package counts and disposition
categories (12 retired: 9 CLI-SURFACE, 1 OPERATOR-SKILL, 2 R-NS-6-dissolved).

**Independent re-derivation.** `grep -n "12 retired\|9 CLI-SURFACE\|OPERATOR-SKILL\|R-NS-6" .sergeant/index.md` returns the exact text: "12 retired — 9 CLI-SURFACE, 1 OPERATOR-SKILL, and the 2 R-NS-6-dissolved `grilling`/`grill-with-docs`." `git diff 3a46b87c..HEAD` on both `docs/icm/re-homing-record-2026-08-12.md` and `.sergeant/index.md` is empty.

**Verdict.** Confirmed.

### F7 — verdict: CONFIRMED

**Claim under review.** The engine capability list in §3.7 is silently one
item stale: `src/domain/workflow.rs` gained a `catalog()` function backing a
new `GET /v1/workflows` route (T-series T2) that postdates the proposal's
audit pin — narrow, does not affect the "no code prerequisite" conclusion
for the ladder work, but the "already supports" enumeration is incomplete
relative to current `main`.

**Independent re-derivation.** `git diff --stat 3a46b87c..HEAD` across the
proposal's own cited files shows exactly one changed file:
`src/domain/workflow.rs | 415 +++...` (zero changes to `engine.rs`,
`claude.rs`). Grepping the diff confirms `catalog()`, `CatalogEntry`,
`WorkflowIndexFrontMatter`, `ROOT_CATALOG_FILE`, and `INDEX_FILE` are all
genuinely new. `src/api.rs:420` confirms `.route("/workflows", get(list_workflows))` is wired under the `/v1` router, i.e. a real new `GET /v1/workflows` route, not dead code. `git log --oneline 3a46b87c..HEAD -- src/domain/workflow.rs src/api.rs` attributes this to T-series commits `2621915` ("T-series T2: workflow discovery") and `c4032ae` ("Merge T2 (workflow discovery)..."), consistent with the critic's "T2-39/T2-40" attribution in spirit (exact ticket IDs weren't independently verifiable from commit messages alone, but the T2/workflow-discovery attribution is correct).

**Attempt to refute.** Checked whether the new route is actually reachable
(not just declared) — confirmed via `api.rs`'s router wiring. Checked
whether it touches stage execution, `needs_input`, or retry, which would
undercut the "unrelated to what the ladder work depends on" claim — it does
not; `catalog()` only reads `index.md`/`workflow.toml` front matter for
listing purposes, no stage-execution interaction found.

**Verdict.** Confirmed, including the critic's qualification that this does
not change ICMR-F7's core "no Rust prerequisite" verdict.

### F8 — verdict: CONFIRMED

**Claim under review.** All six §12 packages (`task-intake-and-route`,
`direct-implementation`, `sergeant-setup`, `dispatch`, `worker-mission`,
`load-project`) exist with the described stage shapes; none was renamed,
retired, or restructured by the T-series sprint.

**Independent re-derivation.** `ls .sergeant/workflows/` confirms all six
directories exist by exact name. `git diff --stat 3a46b87c..HEAD -- .sergeant/index.md skills/ docs/icm/` (the ICM doc set and catalog) shows zero changes outside `src/domain/workflow.rs`, corroborating "no package renamed/retired/restructured."

**Verdict.** Confirmed.

### F9 — verdict: CONFIRMED

**Claim under review.** Exactly 23 workflow directories and 4 skill
directories exist, matching the proposal's "23 workflows, 4 skills" claim
and `.sergeant/index.md`'s own count.

**Independent re-derivation.** `ls -d .sergeant/workflows/*/ | wc -l` → 23.
`ls -d skills/*/ | wc -l` → 4, named `estate-navigation`, `grill-with-docs`,
`grilling`, `sergeant-help` — matching the critic's list exactly.

**Verdict.** Confirmed.

## Summary

Nine of nine findings independently re-derive to the same substance the
critic reported. Eight are unqualified CONFIRMED — every underlying file
read, grep, and diff the critic cited was reproduced from scratch here and
matched. F3 is CONFIRMED on its facts but DOWNGRADED in effect: the critic
rated a fabricated-direct-quotation defect (paraphrase presented as an
exact `>` quote) at the same "info" severity tier as findings with literally
zero diff from the pin, which understates how it differs in kind from a
clean pass, even though it correctly does not undermine §3.3's downstream
argument. No finding is struck. No finding here invalidates a section's
premise or rises to `error` severity; the fidelity axis's overall
"proposal holds up well" conclusion survives adversarial refutation.
