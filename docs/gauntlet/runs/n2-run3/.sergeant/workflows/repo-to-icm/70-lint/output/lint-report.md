# Lint report — `70-lint`

`../60-draft/output/draft-report.md` did not open with
`# AMBIGUOUS — NOT RESOLVED`, so this stage proceeded with its ordinary
work (`../_config/run-discipline.md` §2).

Method: `python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py
<candidate-path>` run against every candidate package named in
`../60-draft/output/draft-report.md`'s manifest (18/18), from the
repository root. Each reported defect classified per
`references/mechanical-vs-substantive.md`; mechanical defects fixed
directly in the candidate package and the validator re-run until no
mechanical defects remained. This workflow's own tree was then validated
with no path argument.

18/18 candidates covered. 11 PASS clean after mechanical repair, 7 FAIL
with a substantive defect left for `80-adversarial-review` and
`90-reconcile`.

---

## Systemic mechanical defect fixed across 5 candidates

Every non-first stage of `dispatch-worker`, `task-intake-and-execution`,
`review-finding-routing`, `validation-gate`, and `worker-response-and-recovery`
carried an identical `Inputs` row:

```
| ../CONTEXT.md | L1 | orientation for this candidate workflow |
```

Classified **mechanical**: `../_config/run-discipline.md` (an Input to
this stage) states plainly that "the engine does not deliver [Layer 1
orientation] to any stage past the first" — the same fact this workflow's
own real, admitted tree already honors (no stage past the first anywhere
under `.sergeant/workflows/repo-to-icm/` cites its own workflow-level
`CONTEXT.md` as an Input). Since L1 is definitionally the workflow's own
top-level orientation file, no other Layer tag correctly describes it for
a non-first stage either — the row is simply not citable there under an
already-established rule, so removing it (rather than retagging it)
requires no new judgment about what any of these candidates should mean.
Fixed by deleting the row from each affected stage's `## Inputs` table;
re-validated clean.

Affected stages: `dispatch-worker/{02-create-td-tasks,03-record-canonical-intent,
04-spawn-worker,05-escalate-undecided-seam,06-report-terminal-status}`;
`task-intake-and-execution/{02-resolve-task,03-reconcile-before-start,
04-execute,05-direct-mode-implementation,06-handle-decision-gate,
07-direct-mode-delivery,08-reconcile-and-deliver}`;
`review-finding-routing/02-reconcile-hand-edit`;
`validation-gate/02-publish-readiness`;
`worker-response-and-recovery/02-respond-to-worker`.

---

## Per-candidate results

### 1. `dispatch-worker`

- **Initial:** FAIL, 5 defects — `[S4]` (see systemic fix above) on stages
  02–06.
- **Classification:** mechanical (systemic fix above).
- **Fixed:** yes, all 5.
- **Final:** PASS — structure is clean.

### 2. `task-intake-and-execution`

- **Initial:** FAIL, 7 defects — `[S4]` (see systemic fix above) on stages
  02–08.
- **Classification:** mechanical (systemic fix above).
- **Fixed:** yes, all 7.
- **Final:** PASS — structure is clean.

### 3. `worker-response-and-recovery`

- **Initial:** FAIL, 1 defect — `[S4]` (see systemic fix above) on stage
  02-respond-to-worker.
- **Classification:** mechanical (systemic fix above).
- **Fixed:** yes.
- **Final:** PASS — structure is clean.
- **Note for `90-reconcile`:** unrelated to this stage's lint scope, but
  `../60-draft/output/draft-report.md` records that this candidate's own
  `recover-worker` checkpoint (five stage-context records, no attached
  `representation: stage` record) is intentionally not materialized as a
  stage directory — carried forward here only as context for why this
  package has 2 stages, not more.

### 4. `validation-gate`

- **Initial:** FAIL, 1 defect — `[S4]` (see systemic fix above) on stage
  02-publish-readiness.
- **Classification:** mechanical (systemic fix above).
- **Fixed:** yes.
- **Final:** PASS — structure is clean.

### 5. `project-registration`

- **Initial:** PASS — structure is clean.
- **Classification:** n/a — no defects reported.
- **Fixed:** n/a.
- **Final:** PASS — structure is clean.

### 6. `fleet-monitor-and-reconcile`

- **Initial:** FAIL, 1 defect — `[S3] fleet-monitor-and-reconcile: workflow.toml has no non-empty `workflow.stages` array`.
- **Classification:** **substantive.** `../60-draft/output/draft-report.md`
  ("Method notes on two judgment calls this stage made" → "Zero-stage
  candidates") records that this candidate deliberately materializes with
  `stages = []` per `../../_config/icm-ladder.md` bucket 3, rather than
  inventing a checkpoint to give it a conventional shape. Repairing this
  finding would mean either inventing a stage (exactly the invention this
  run's own discipline forbids) or overriding the validator's assumption
  that every workflow has ≥1 stage — both are judgment calls about how the
  candidate should be shaped, not "making the file agree with an
  already-established fact," so this is not a mechanical repair available
  to this stage.
- **Fixed:** no — left for `80-adversarial-review`/`90-reconcile`.
- **Final:** FAIL — 1 defect remaining: `[S3]` as above.

### 7. `shipping-gate-driving`

- **Initial:** PASS — structure is clean.
- **Classification:** n/a — no defects reported.
- **Fixed:** n/a.
- **Final:** PASS — structure is clean.

### 8. `review-finding-routing`

- **Initial:** FAIL, 1 defect — `[S4]` (see systemic fix above) on stage
  02-reconcile-hand-edit.
- **Classification:** mechanical (systemic fix above).
- **Fixed:** yes.
- **Final:** PASS — structure is clean.

### 9. `installation-and-setup`

- **Initial:** PASS — structure is clean.
- **Classification:** n/a — no defects reported.
- **Fixed:** n/a.
- **Final:** PASS — structure is clean.

### 10. `project-graphify`

- **Initial:** PASS — structure is clean.
- **Classification:** n/a — no defects reported.
- **Fixed:** n/a.
- **Final:** PASS — structure is clean.

### 11. `fleet-cleanup`

- **Initial:** PASS — structure is clean.
- **Classification:** n/a — no defects reported.
- **Fixed:** n/a.
- **Final:** PASS — structure is clean.

### 12. `dag-orchestration`

- **Initial:** PASS — structure is clean.
- **Classification:** n/a — no defects reported.
- **Fixed:** n/a.
- **Final:** PASS — structure is clean.

### 13. `callback-delivery`

- **Initial:** FAIL, 1 defect — `[S3] callback-delivery: workflow.toml has no non-empty `workflow.stages` array`.
- **Classification:** **substantive** — same zero-stage reasoning as
  `fleet-monitor-and-reconcile` above (`../60-draft/output/draft-report.md`
  "Zero-stage candidates"; this candidate is explicitly named in that
  list).
- **Fixed:** no — left for `80-adversarial-review`/`90-reconcile`.
- **Final:** FAIL — 1 defect remaining: `[S3]` as above.

### 14. `skill-adoption`

- **Initial:** FAIL, 1 defect — `[S3] skill-adoption: workflow.toml has no non-empty `workflow.stages` array`.
- **Classification:** **substantive** — same zero-stage reasoning
  (explicitly named in `../60-draft/output/draft-report.md` "Zero-stage
  candidates").
- **Fixed:** no — left for `80-adversarial-review`/`90-reconcile`.
- **Final:** FAIL — 1 defect remaining: `[S3]` as above.

### 15. `sergeant-help-query`

- **Initial:** FAIL, 1 defect — `[S3] sergeant-help-query: workflow.toml has no non-empty `workflow.stages` array`.
- **Classification:** **substantive** — same zero-stage reasoning
  (explicitly named in `../60-draft/output/draft-report.md` "Zero-stage
  candidates").
- **Fixed:** no — left for `80-adversarial-review`/`90-reconcile`.
- **Final:** FAIL — 1 defect remaining: `[S3]` as above.

### 16. `troubleshoot-td-identity`

- **Initial:** FAIL, 1 defect — `[S3] troubleshoot-td-identity: workflow.toml has no non-empty `workflow.stages` array`.
- **Classification:** **substantive** — same zero-stage reasoning
  (explicitly named in `../60-draft/output/draft-report.md` "Zero-stage
  candidates").
- **Fixed:** no — left for `80-adversarial-review`/`90-reconcile`.
- **Final:** FAIL — 1 defect remaining: `[S3]` as above.

### 17. `cross-repo-planning`

- **Initial:** FAIL, 1 defect — `[S3] cross-repo-planning: workflow.toml has no non-empty `workflow.stages` array`.
- **Classification:** **substantive** — same zero-stage reasoning
  (explicitly named in `../60-draft/output/draft-report.md` "Zero-stage
  candidates").
- **Fixed:** no — left for `80-adversarial-review`/`90-reconcile`.
- **Final:** FAIL — 1 defect remaining: `[S3]` as above.

### 18. `undocumented-failure-escalation`

- **Initial:** FAIL, 1 defect — `[S3] undocumented-failure-escalation: workflow.toml has no non-empty `workflow.stages` array`.
- **Classification:** **substantive** — same zero-stage reasoning
  (explicitly named in `../60-draft/output/draft-report.md` "Zero-stage
  candidates").
- **Fixed:** no — left for `80-adversarial-review`/`90-reconcile`.
- **Final:** FAIL — 1 defect remaining: `[S3]` as above.

---

## Repository-wide (not attributable to any one candidate)

No `[S7]` findings surfaced against any candidate in this run (checked:
none of the 18 per-candidate runs, nor the no-argument self-check below,
reported an `[S7]` code). This heading is included for completeness per
this stage's own `CONTEXT.md`; there is nothing to record under it this
run.

---

## This workflow's own tree

Ran `python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py`
with no argument (admitted mode, validates `.sergeant/workflows/repo-to-icm`
as the parent of `scripts/`).

```
validated: .sergeant/workflows/repo-to-icm  (mode=admitted)
engine-gap records checked: 2

PASS: structure is clean
```

**Final: PASS.** No `[S9]` engine-gap defects found in
`../40-classify/output/classifications.ndjson` (the validator's
repository-wide NDJSON scan covers any `.ndjson` under this tree,
including that file, and reported none) — no substantive engine-gap
finding to record for this workflow's own tree.

---

## Summary

| Candidate | Initial | Fixed (mechanical) | Final | Substantive remaining |
|---|---|---|---|---|
| dispatch-worker | FAIL (5×S4) | 5 | PASS | — |
| task-intake-and-execution | FAIL (7×S4) | 7 | PASS | — |
| worker-response-and-recovery | FAIL (1×S4) | 1 | PASS | — |
| validation-gate | FAIL (1×S4) | 1 | PASS | — |
| project-registration | PASS | 0 | PASS | — |
| fleet-monitor-and-reconcile | FAIL (1×S3) | 0 | FAIL | 1×S3 (zero-stage design) |
| shipping-gate-driving | PASS | 0 | PASS | — |
| review-finding-routing | FAIL (1×S4) | 1 | PASS | — |
| installation-and-setup | PASS | 0 | PASS | — |
| project-graphify | PASS | 0 | PASS | — |
| fleet-cleanup | PASS | 0 | PASS | — |
| dag-orchestration | PASS | 0 | PASS | — |
| callback-delivery | FAIL (1×S3) | 0 | FAIL | 1×S3 (zero-stage design) |
| skill-adoption | FAIL (1×S3) | 0 | FAIL | 1×S3 (zero-stage design) |
| sergeant-help-query | FAIL (1×S3) | 0 | FAIL | 1×S3 (zero-stage design) |
| troubleshoot-td-identity | FAIL (1×S3) | 0 | FAIL | 1×S3 (zero-stage design) |
| cross-repo-planning | FAIL (1×S3) | 0 | FAIL | 1×S3 (zero-stage design) |
| undocumented-failure-escalation | FAIL (1×S3) | 0 | FAIL | 1×S3 (zero-stage design) |
| **this workflow's own tree** | PASS | 0 | PASS | — |

18/18 candidates covered, 15 mechanical defects fixed (all `[S4]`, one
systemic pattern), 7 substantive `[S3]` defects left standing across the
7 candidates `../60-draft/output/draft-report.md` designed as
deliberately zero-stage — real signal for `80-adversarial-review` and
`90-reconcile`, not a failure of this stage.
