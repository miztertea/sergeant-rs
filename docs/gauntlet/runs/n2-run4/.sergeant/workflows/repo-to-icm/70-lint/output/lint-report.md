# 70-lint — lint report (N2 run, 21-partition corpus, 44-candidate draft tree)

`../60-draft/output/draft-report.md` does not open with `# AMBIGUOUS — NOT RESOLVED`, so this stage proceeded with its ordinary work (`../_config/run-discipline.md` §2 checked, not triggered).

**Scope.** All **44** candidate packages named in `../60-draft/output/draft-report.md` §1's manifest (this run's corpus — 21 harvested partitions — produced 44 candidates, versus run 3's 6-partition corpus) were run through `.sergeant/workflows/repo-to-icm/scripts/validate-structure.py`, plus this workflow's own tree (no-argument run). None were sampled, capped, or skipped for volume; the manifest's candidate list is the bound this stage's method licenses (`CONTEXT.md` "For each candidate package path from `../60-draft/output/draft-report.md`"), and every row in it got a full validator run, a classification pass, and its own heading below.

**Headline result.** Every one of the 44 candidates fails the validator with the same single defect, `[S12]` (closing-stage finalize step not named), and no other defect code appears anywhere in this run — 0 `[S1]`–`[S11]`, `[S13]`–`[S15]` findings across all 44 runs, 0 `[S7]` repository-wide findings. That `[S12]` finding is classified **substantive** below (§ "Systemic finding") and is therefore left unfixed for `80-adversarial-review`/`90-reconcile`, per `references/mechanical-vs-substantive.md`'s instruction not to force a mechanical-looking fix onto a substantive defect. Because the one defect present in every candidate is substantive, **zero mechanical repairs were made this run** — not because repair was skipped, but because none of the 44 initial validator runs surfaced a mechanical defect to repair. Re-running the validator after a no-op repair pass reproduces the identical initial result for every candidate, so the "final validator result" recorded per candidate below is identical to its "initial validator result" throughout.

## Systemic finding: `[S12]` closing-stage finalize step (all 44 candidates)

**What the validator reports**, per candidate (`<stage>` is that candidate's own last-in-order stage directory, identified in the table below):

```
[S12] <candidate>: outputs are declared but the closing stage `<stage>` names no
      finalize step (docs/icm/convention.md §1a, D9)
```

**Root cause (checked, not assumed):** `.sergeant/workflows/repo-to-icm/60-draft/references/draft-package-template.md` — the template `60-draft` used to materialize all 44 packages — never mentions "finalize" anywhere in its own text (`grep -i finalize` on it returns nothing). Every closing stage's `CONTEXT.md` was materialized faithfully to that template, so every one of them omits finalize-step language for the same structural reason. This is not 44 independent typos; it is one template gap surfacing 44 times.

**Classification: substantive**, per `references/mechanical-vs-substantive.md`'s test ("repairing it requires no new judgment ... only making the file agree with a fact already established elsewhere in this run's own artifacts"):

- There is no fact already established elsewhere in this run's own artifacts that says what any given candidate's finalize step *does* — no upstream artifact (`draft-report.md`, a candidate's own `provenance.md`, `40-classify`'s `classifications.ndjson`) states a deterministic finalize action for these closing stages. Writing one means authoring new process description for that specific workflow — exactly the "inventing missing evidence" / "deciding something this stage was not given the authority to decide" the substantive column names.
- The validator's own check (`check_finalize_step` in `validate-structure.py`) is a bare case-insensitive substring test for the word "finalize" anywhere in the closing stage's `CONTEXT.md`. Satisfying only the string match (e.g. appending a sentence that merely contains the word without describing a real deterministic step) would be precisely the "mechanical-looking fix onto a substantive defect" `references/mechanical-vs-substantive.md` and `../CONTEXT.md` both instruct against — it would hide the real gap from `80`/`90` while adding no actual meaning.
- It is not in the explicit Mechanical list (name/directory agreement, ordering/typo drift, Layer-tag correction, Inputs-path typo, stray executable bit, obvious Disposition line, formatting) and matches the shape of the explicit Substantive list's "stage whose Inputs table is missing a row entirely ... asserts a new dependency claim this stage cannot verify on its own" — here, a new *capability claim* ("this stage deterministically finalizes the workflow, thus") rather than a new dependency, but the same kind of authority this stage was not given.

**Disposition:** logged once here as the systemic finding; not fixed. It is real signal for `80-adversarial-review` and `90-reconcile`: `60-draft`'s own template (Layer 3, stable across runs) is missing finalize-step guidance for every generated closing stage, and that is a template-level fix outside this stage's authority to make unilaterally on 44 packages' behalf.

## Per-candidate results

Same shape for all 44: one `validate-structure.py <path>` run, one `[S12]` finding (closing stage named below), classified substantive per the systemic finding above, no mechanical defects found, no repair applied, re-run not needed (nothing changed) — final result equals initial result.

### `adopt-external-skill`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-vet-and-adopt-skill` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-vet-and-adopt-skill`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `callback-protocol`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `07-retry-delivery` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `07-retry-delivery`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `check-repo-status`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-report-repo-status` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-report-repo-status`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `ci-verification`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-verify-bash-compat-both-passes` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-verify-bash-compat-both-passes`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `code-review`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `03-aggregate-review-report` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `03-aggregate-review-report`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `cross-repo-work`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `04-reconcile-cross-repo-outcome` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `04-reconcile-cross-repo-outcome`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `dag-run`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `04-run-dispatch-hook` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `04-run-dispatch-hook`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `design-it-twice`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `03-compare-and-recommend` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `03-compare-and-recommend`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `diagnose-bug`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `05-declare-bug-fixed` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `05-declare-bug-fixed`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `direct-mode`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `04-deliver` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `04-deliver`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `dispatch-mode`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `19-detect-model-substitution` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `19-detect-model-substitution`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `domain-modeling`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `02-offer-adr` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `02-offer-adr`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `fleet-status-listing`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-list-fleet-status` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-list-fleet-status`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `graphify`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `02-recover-from-failed-publish` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `02-recover-from-failed-publish`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `grilling`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-conduct-interview` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-conduct-interview`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `implement`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `03-commit-implementation` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `03-commit-implementation`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `install-sergeant`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `04-update-checkout` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `04-update-checkout`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `invoke-grill-with-docs`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-run-grill-with-docs` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-run-grill-with-docs`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `list-projects`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-list-projects` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-list-projects`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `list-tasks`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-list-tasks` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-list-tasks`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `load-project`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `03-edit-and-validate-project` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `03-edit-and-validate-project`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `no-mistakes-finding-routing`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `02-remediate-grouped-findings` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `02-remediate-grouped-findings`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `notify-primary-session`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `02-capture-wiki-activity` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `02-capture-wiki-activity`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `prototype`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `07-drive-ui-prototype` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `07-drive-ui-prototype`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `record-recovery-pointer`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-bind-worktree-identity` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-bind-worktree-identity`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `register-project`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-validate-and-register-project` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-validate-and-register-project`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `research`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-conduct-research` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-conduct-research`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `resolve-merge-conflict`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `03-complete-merge` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `03-complete-merge`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `review-findings-routing`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `03-publish-blocked-gate` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `03-publish-blocked-gate`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `sergeant-help`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `02-handle-failure-or-handoff` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `02-handle-failure-or-handoff`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `sergeant-setup`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `10-phase9-graphify-init` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `10-phase9-graphify-init`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `standard-workflow`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `08-deliver-and-cleanup` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `08-deliver-and-cleanup`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `sync-project-repos`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `02-clone-missing-repo` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `02-clone-missing-repo`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `tdd`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `02-run-red-green-loop` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `02-run-red-green-loop`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `to-spec`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `03-publish-spec` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `03-publish-spec`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `to-tickets`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `06-report-dispatch-frontier` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `06-report-dispatch-frontier`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `treehouse-init`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-initialize-treehouse` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-initialize-treehouse`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `triage`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `08-quick-override` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `08-quick-override`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `troubleshoot-failure`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `01-escalate-undocumented-gap` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `01-escalate-undocumented-gap`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `validation-pipeline-gate`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `14-monitor-active-run` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `14-monitor-active-run`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `wayfinder`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `04-work-through-map-session` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `04-work-through-map-session`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `wiki-maintenance`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `02-schedule-wiki-digest` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `02-schedule-wiki-digest`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `worker-contract`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `02-report-terminal-status` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `02-report-terminal-status`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

### `worker-lifecycle`

- **Initial validator result:** FAIL — 1 defect: `[S12]` — outputs are declared but the closing stage `26-stop-background-monitor` names no finalize step (docs/icm/convention.md §1a, D9)
- **Classification:** substantive (systemic finding above) — closing stage `26-stop-background-monitor`
- **Mechanical defects found/fixed:** none
- **Final validator result:** FAIL — same 1 defect (`[S12]`), left for `80`/`90`

## Repository-wide (not attributable to any one candidate)

No `[S7]` finding occurred in any of the 44 runs (each run's `check_no_misplaced_drafts` repository-wide scan — comparing `.sergeant/workflows/` against `.sergeant/drafts/workflows/` — came back clean every time: nothing under `.sergeant/workflows/` declares `status: draft`, nothing under `.sergeant/drafts/workflows/` declares `status: published`). Nothing to record under this heading this run.

## This workflow's own tree

`python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py` (no argument, mode=admitted) against `.sergeant/workflows/repo-to-icm`:

```
validated: .sergeant/workflows/repo-to-icm  (mode=admitted)
engine-gap records checked: 0

PASS: structure is clean
```

**Result: PASS.** No mechanical or substantive defects found; no repair needed.

This run's own tree is *not* assumed clean by construction — the no-argument run was executed and its result recorded, per `../CONTEXT.md`'s instruction that a run worktree (unlike the authored tree) carries `40-classify`'s freshly-written `classifications.ndjson` and must actually be scanned by the validator's `[S9]` engine-gap check, not skipped.

**`[S9]` engine-gap check, specifically:** `engine-gap records checked: 0`. Independently confirmed by direct inspection of `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (1,333 records this run): `grep -c '"representation"\s*:\s*"engine-gap"'` returns `0`. The record's `representation` values this run are `stage-context` (795), `helper` (207), `stage` (174), `agents-invariant` (126), `shared-helper` (23), `workflow` (8) — no `engine-gap` records exist to have a missing field. This is a genuine absence of the defect class, not the validator failing to look: the walk covers every `.ndjson` file under the tree, and `40-classify/output/classifications.ndjson` is under it.

