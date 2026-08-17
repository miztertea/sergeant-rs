# 60-close-out: close out

## Inputs

| File | Layer | Why |
|---|---|---|
| ../50-reconcile-custody/output/README.md | L4 | upstream artifact produced by `50-reconcile-custody` |

## Purpose

Stop driving at `checks-passed`; on `failed`/`cancelled`, fix on the same branch and re-drive; summarize what the pipeline found and fixed.

Trigger (workflow-level): Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## What must become true here (durable outcome)

Stop driving at `checks-passed`; on `failed`/`cancelled`, fix on the same branch and re-drive; summarize what the pipeline found and fixed.

## Behavior contract

- **`checks-passed` means the change is validated and CI is green but the PR is not yet merged; the actor is done driving the pipeline and should tell the user the PR is ready to review and merge (link is in the `help` line) without waiting for the merge, since no-mistakes keeps monitoring the PR in the background until merged, closed, or idle-timed-out.**
  (trigger: the pipeline reaches the checks-passed outcome; outcome: the actor stops driving and hands the merge decision to the user, while the pipeline's own monitor continues in the background)
  — `BU-P2-086`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Outcome: checks-passed, lines 169-176)
- **`passed` means the changes cleared the gate and the PR was merged or closed.**
  (trigger: the pipeline reaches the passed outcome; outcome: the workflow's fully-terminal success state is reached)
  — `BU-P2-087`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Outcome: passed, lines 177-177)
- **`failed` or `cancelled` mean the change did not clear the gate; the actor reads the output, fixes whatever is pointed at (a failing test, lint error, or a skipped finding), commits the fix on the same feature branch, and drives the pipeline again with a fresh `axi run --intent` or `no-mistakes rerun`; the actor must not leave the user at a failed outcome without either retrying or explaining what blocks it.**
  (trigger: the pipeline reaches a failed or cancelled outcome; outcome: the actor either retries after a concrete fix or explicitly explains the blocker to the user — never silence)
  — `BU-P2-088`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Outcomes: failed/cancelled, lines 178-186)
- **The CI step deliberately keeps watching the PR after checks pass, so `axi run` returns `checks-passed` as soon as checks are green rather than blocking on the human merge; the actor must never poll or re-run waiting for the merge.**
  (trigger: checks have passed but the PR is not yet merged; outcome: the actor does not poll or loop waiting for a human merge; the pipeline's own monitor covers it)
  — `BU-P2-095`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (CI monitor stays live, lines 198-200)
- **A PR that falls behind the default branch or hits a merge conflict after checks pass needs no command from the actor and must never be hand-rebased: the still-running CI monitor detects an actual conflict, rebases onto the base, resolves it, and re-pushes itself; a merely-behind-but-clean PR needs nothing since the platform merges it; the one exception is when the monitor is no longer running (PR closed, run aborted/superseded, idle-timed-out, or auto-fix attempts exhausted), in which case the actor recovers with `no-mistakes rerun`, which cancels the stale monitor and re-runs the full pipeline including a deterministic rebase step; `no-mistakes axi run` must not be used to refresh a still-active PR, since after checks-passed it just reattaches to the running monitor without rebasing.**
  (trigger: a PR has passed checks and later drifts behind the base or conflicts; outcome: drift/conflict is resolved by the pipeline's own live monitor automatically; the actor intervenes only when that monitor has stopped, and then via `rerun`, never a hand-rebase)
  — `BU-P2-096`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (PR drift handling, lines 202-214)
- **On a successful outcome (`checks-passed` or `passed`), the actor closes the loop with the user by summarizing what happened during the pipeline concisely and readably — what was validated and what was found — and if the output includes a `fixes` table, explicitly acknowledges and lists each fix the pipeline made that the actor's original change missed.**
  (trigger: the pipeline reaches a successful terminal or near-terminal outcome; outcome: the user receives an honest, itemized summary including fixes the pipeline had to make)
  — `BU-P2-097`, `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md` (Success closeout, lines 216-221)
- **Stop driving at checks-passed: the PR is ready and no-mistakes monitors it in the background, so do not poll or wait for merge.**
  (trigger: a run reaches checks-passed; outcome: coordinator involvement ends at checks-passed rather than continuing through merge)
  — `BU-P1-077`, `reference/sergeant-upstream/README.md` (README.md L293)
- **Remediation that changes HEAD still requires independent rereview before updating the readiness marker, but must not trigger repeated no-mistakes review cycles.**
  (trigger: remediation changes HEAD after an initial validation pass; outcome: rereview happens exactly once per remediation without re-running the whole gate)
  — `BU-P1-043`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L155-157)

## Helper: handover log (folded from demoted `90-handover-log`, N1 adjudication A4)

`90-handover-log` was classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 it is demoted and its behavior folded here as the concluding helper invocation of this checkpoint — any coordinator ownership transfer that occurred during the run is durably logged as this stage closes out. Statements below are already normalized per N1 adjudication A11 (the source's "pane" wording is old-Sergeant tmux mechanism; this project's durable execution/session identity replaces it — deviation register D2, `reference-corpus/synthesis.md` §4 clusters M1-M4):

- **Every ownership transfer is durably appended to an owner-only handover log recording timestamp, reason, repository, prior and new session, and both identity tuples, and a release token is consumed by the claim that uses it so it can never be replayed by a third pane later.**
  (trigger: an ownership claim or release occurs; outcome: every transfer is durably auditable and a release can never be reused by an unintended third party)
  — `BU-P8-089`, `reference/sergeant-upstream/docs/using-sergeant.md` (L376-379)
- **Coordinator-owned validation must distinguish its own diagnostics from a worker's, and must verify a handed-over coordinator identity before proceeding, so validation ownership (who is allowed to run/approve no-mistakes) is never ambiguous between a worker and the coordinator.**
  (trigger: coordinator-owned no-mistakes validation is launched, possibly after a pane handover; outcome: validation authority is unambiguous and independently verified, matching the worker-brief's own rule that the coordinator owns every no-mistakes gate and a worker pane may never approve or route one)
  — `BU-P7-104`, `reference/sergeant-upstream/tests/sgt-validate-test.sh` (line 1180)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Distinguishing `checks-passed`/`passed`/`failed`/`cancelled` and taking the matching path — stop and hand off, fix and re-drive, or explain the blocker (`BU-P2-086`–`088`).
- Summarizing what the pipeline found and fixed for the user (`BU-P2-097`).

### J1 — local choices allowed
- Summary wording, provided every pipeline-made fix is explicitly acknowledged (`BU-P2-097`).

### J0 — must become `needs_input`
- A `failed`/`cancelled` outcome whose blocker cannot be concretely fixed or concretely explained — silence is never an option (`BU-P2-088`).
- This stage's own `checks-passed`/`passed` outcomes presuppose a PR already exists and CI already ran — this stage does not itself open that PR or trigger that CI, but that is expected, ordinary behavior, not an unresolved authority gap (`BU-VAS-15`, resolved — see the workflow-level `CONTEXT.md`'s "Resolved" note). If this stage is ever reached at `checks-passed`/`passed` without a PR/CI having actually happened, that is itself a `J0` condition, not something to paper over by describing the expected happy path as if it occurred.

### Completion boundary
This stage may complete only at a terminal or near-terminal pipeline outcome, with the user given an honest, itemized summary — never left at a failed outcome in silence.

### Decision evidence
The outcome acted on and the summary given are this stage's own decision record; ownership-transfer logging (below) is separate and mechanical.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
