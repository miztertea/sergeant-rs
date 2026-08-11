# provenance — dispatch-mode

Maps every stage, and the workflow as a whole, to the `behavior_id`(s) in
`../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson`
that justify it, per `../../../workflows/repo-to-icm/60-draft/references/
draft-package-template.md`.

## Workflow as a whole

- **`behavior_id`:** `BU-0003`
- **Representation:** `workflow`
- **Basis:** direct — `BU-0003` is an explicit `representation: workflow`
  record naming `dispatch-mode`, carrying its trigger ("work spans
  repositories, has independent repo-owned sub-tasks, needs isolated
  review, or the user explicitly requests workers"), outcome ("one worker
  is dispatched per owning repository"), and completion condition
  ("progress is monitored through to reconciliation of merge order and
  cross-repo implications").

## Stages

### `10-dispatch-worker`

- **`behavior_id`:** `BU-0041`
- **Representation:** `stage`
- **Basis:** direct — `BU-0041` is a `representation: stage` record with
  `workflow: dispatch-mode`, `stage: dispatch-worker`. Trigger: dispatch is
  invoked for one or more repos. Outcome: each targeted repo gets its own
  isolated checkout, a written brief, and a spawned interactive agent
  session.

## Uncovered ground (not a stage, not omitted — recorded)

`BU-0003`'s own outcome text names monitoring and reconciliation of merge
order and cross-repo implications as part of dispatch mode's completion
condition, but no `behavior_id` in
`../../../workflows/repo-to-icm/40-classify/output/classifications.ndjson`
carries `workflow: dispatch-mode` for a monitor or reconcile checkpoint —
`10-dispatch-worker` is the only member stage this corpus classified. This
is not a design inference filling a gap; it is an honest absence, left
unfilled per `../../../workflows/repo-to-icm/60-draft/CONTEXT.md` step 3
("a stage with no direct source evidence is marked as a justified design
inference... never left silent and never given an invented citation") —
inverted here: this is a *workflow-level outcome clause* with no
corresponding stage evidence at all, so no stage is invented to cover it.
A human reviewer promoting this candidate would need to either define a
monitor/reconcile stage from other evidence or narrow the workflow's own
completion condition to match what `10-dispatch-worker` alone delivers.
