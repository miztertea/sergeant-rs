# 50-reconcile-custody: reconcile custody

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-drive-gates/output/README.md | L4 | upstream artifact produced by `40-drive-gates` (folds the demoted `60-route-findings` checkpoint, N1 adjudication A4) |

## Purpose

The structured branch-sync state is processed rather than improvised: sync / continue / recover-custody, never reset, stash, force or branch replacement.

Trigger (workflow-level): Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## What must become true here (durable outcome)

The structured branch-sync state is processed rather than improvised: sync / continue / recover-custody, never reset, stash, force or branch replacement.

## Behavior contract

- **Before any post-pipeline local commit or fresh run, the actor reads the structured `branch_sync` object returned by AXI home/status/drive results, and runs `no-mistakes axi sync` only when its `next_action.code` is `sync`.**
  (trigger: about to make a local commit or start a fresh run after a pipeline interaction; outcome: branch synchronization is checked and, if indicated, performed before any local action)
- **A guarded sync may be a strict fast-forward or a content-equivalent diverged advance that anchors the pre-sync head before moving the branch with reset semantics; genuine divergence stays blocked.**
  (trigger: performing `axi sync`; outcome: only fast-forward or provably content-equivalent advances are applied automatically; real divergence is refused)
- **If `next_action.code` is `continue_active_run`, the pipeline still owns the branch: the actor runs the reported command, keeps driving the active run, and makes no local follow-up commits.**
  (trigger: branch_sync reports continue_active_run; outcome: the actor defers entirely to the still-active pipeline run rather than acting locally)
- **If `next_action.code` is `recover_custody`, a terminal run left unpublished pipeline commits preserved in the local gate; the actor runs `no-mistakes axi sync --recover` to return custody and fast-forward to the preserved head, or `no-mistakes rerun` to resume validating it instead.**
  (trigger: branch_sync reports recover_custody; outcome: the actor either recovers the preserved pipeline commits or resumes validation of them, never discards them silently)
- **A dirty or diverged worktree makes recovery refuse with explicit choices; `--keep-local` keeps the actor's current head while the preserved commits stay anchored under `refs/no-mistakes/recover/<run>`; when synchronization is blocked the actor must process the structured state rather than improvising reset, stash, merge, rebase, force, or branch replacement.**
  (trigger: synchronization or recovery is blocked by worktree state; outcome: the actor uses the offered structured choices rather than any manual git surgery)
- **After synchronization, the actor commits the follow-up on top and re-runs `no-mistakes axi run --intent "..."` with the original user intent, which preserves every prior gate-fix commit regardless of its configured subject.**
  (trigger: synchronization has completed; outcome: follow-up work is committed on top and validation resumes with the original intent, without losing prior pipeline commits)
- **Never improvise a reset, stash, force-push, or branch replacement around a blocked sync state.**
  (trigger: branch_sync is blocked; outcome: recovery only ever uses the three named remediation paths, never an improvised git operation)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Reading the structured `branch_sync` object and dispatching to the correct named path (`sync`, `continue_active_run`, `recover_custody`) per its `next_action.code`.
- Choosing `axi sync --recover` versus `no-mistakes rerun` when `recover_custody` applies.

### J1 — local choices allowed
- None beyond ordinary tool mechanics — every response to `branch_sync` is a named, structured path (J2); nothing here is a local improvisation.

### J0 — must become `needs_input`
- **A dirty or diverged worktree blocks recovery and the offered structured choices (`--keep-local` or the anchored `refs/no-mistakes/recover/<run>` ref) don't resolve which to pick without more context.**
- Any temptation to reset, stash, force-push, or replace the branch to escape a blocked sync — never an available response; if the structured choices don't cover the situation, that is itself the `needs_input` condition.

### Completion boundary
This stage may complete only when the branch-sync state has been processed through one of its named structured paths, never an improvised git operation.

### Decision evidence
The `next_action.code` acted on and the chosen recovery path (where applicable) are this stage's own decision record; no separate file.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
