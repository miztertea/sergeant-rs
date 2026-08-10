# provenance — standard-task-workflow

Maps every stage, and the workflow as a whole, to the `behavior_id`(s) in
`../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson`
that justify it, per `../../../workflows/repo-to-icm/60-draft/references/
draft-package-template.md`.

## Workflow as a whole

- **`behavior_id`s:** `BU-0011`, `BU-0012`, `BU-0013`, `BU-0016`, `BU-0019`
  (the five `stage` records naming `workflow: standard-task-workflow`),
  plus `BU-0014`, `BU-0015` (two unattached `stage-context` records naming
  the same `workflow` value — see "Unattached ground" below).
- **Representation:** none of these carries `representation: workflow` —
  no direct `workflow` record for `standard-task-workflow` exists anywhere
  in this corpus.
- **Basis:** indirect, but evidenced, not a design inference. Per
  `../../../workflows/repo-to-icm/50-synthesize/output/candidates.md`
  bucket 1: "a distinct `workflow` value seen only through
  `stage`/`stage-context` records still earns one candidate." The five
  `stage` records above share a common trigger text ("the user brings a
  task") across `BU-0011`, `BU-0012`, `BU-0013`, and the workflow's
  outcome and completion-condition text is assembled from those same
  records' own trigger/outcome fields — every clause traces to a specific
  member record, none is invented.

## Stages

### `10-load-context`

- **`behavior_id`:** `BU-0011`
- **Representation:** `stage`
- **Basis:** direct. Scope: "standard task workflow, step 1 (Load
  context)". Trigger: the user brings a task. Outcome: execution-mode
  selection is made only after context is loaded.

### `20-check-queue`

- **`behavior_id`:** `BU-0012`
- **Representation:** `stage`
- **Basis:** direct. Scope: step 2. Trigger: the user brings a task.
  Outcome: an existing canonical task is reused rather than a duplicate
  created.

### `30-reconcile-existing-state`

- **`behavior_id`:** `BU-0013`
- **Representation:** `stage`
- **Basis:** direct. Scope: step 4. Trigger: the user brings a task.
  Outcome: preserved in-flight work is resumed/taken over instead of
  duplicated.

### `40-validate`

- **`behavior_id`:** `BU-0019`
- **Representation:** `stage`
- **Basis:** direct, but with an unresolved ordering detail. Scope:
  "validation-boundary execution" — **no step number given** (unlike the
  other four member stages, numbered 1, 2, 4, 9). Trigger: a worker
  reaches readiness, or remediation changes HEAD after readiness. Outcome:
  validation runs exactly once as a dedicated boundary, and post-readiness
  HEAD changes get rereview without retriggering the full cycle.
  `50-synthesize`'s own recorded reasoning for placing it between steps 4
  and 9 (verbatim from `candidates.md`, not re-derived here): "I placed
  `validate` between `reconcile-existing-state` (step 4) and
  `reconcile-and-deliver` (step 9) because its trigger ('a worker reaches
  readiness') can only fire after work has been dispatched and monitored —
  occurring later than step 4 — and its outcome must be settled before
  `reconcile-and-deliver`'s own trigger ('a task appears to have reached
  completion') can honestly fire — occurring before step 9. I cannot pin an
  exact step number (5, 6, 7, or 8) for it from the corpus alone; this is a
  judgment call `80-adversarial-review` should be free to challenge."

### `50-reconcile-and-deliver`

- **`behavior_id`:** `BU-0016`
- **Representation:** `stage`
- **Basis:** direct. Scope: step 9. Trigger: a task appears to have
  reached completion. Outcome: cleanup never runs ahead of verified
  terminal state and preserved evidence.

## Unattached ground (not stages, not omitted — recorded)

Two `stage-context` records name this workflow but attach to no stage
candidate here (per `../../../workflows/repo-to-icm/50-synthesize/output/
candidates.md`'s "Unattached records" section, verified there
programmatically against the five `(workflow, stage)` keys above):

- `BU-0014` — `stage: confirm-decisions`, scope step 5. No `stage`
  candidate named `confirm-decisions` exists under `standard-task-workflow`
  in this corpus.
- `BU-0015` — `stage: monitor-progress`, scope step 7. No `stage`
  candidate named `monitor-progress` exists under `standard-task-workflow`
  in this corpus.

Per `../../../workflows/repo-to-icm/60-draft/CONTEXT.md` step 2, this is
guidance content that would attach to its named stage's own `CONTEXT.md`
if that stage existed — since neither `confirm-decisions` nor
`monitor-progress` is a member stage of this candidate, neither is
attached here. No stage candidate is invented to hang them on. This
explains the step-number gaps (3, 5, 6, 7, 8) in the ordering above: steps
5 and 7 are these two unattached records; steps 3, 6, and 8 have no
classified evidence at all in this corpus.
