# 02-drive-gate-findings

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the validation pipeline gate presents findings of a given disposition

**Outcome:** each disposition is handled by its own fixed rule, with ask-user always requiring a human decision

**Statement (the operative rule):** At each validation pipeline gate: `auto-fix` findings are authorized selectively via the pipeline-automation tool after review; `ask-user` findings are relayed to the user and never approved/fixed/skipped autonomously; `no-op` findings are informational and the gate is approved.

## What must become true here (durable outcome)

Each disposition is handled by its own fixed rule, with ask-user always requiring a human decision — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0083`: Running or responding to the pipeline-automation tool blocks while work is active — a quiet step is not a stall — and progress is checked against the validation pipeline's status without issuing duplicate run commands.
- `BU-0085`: While the validation pipeline run is active: the pipeline-owned worktree is never edited, the run is never aborted or rerun to escape a gate, and all pipeline-created commits are preserved; abort is used only when intentionally discarding the entire run.
- `BU-0086`: Driving stops at `checks-passed`; the PR is ready and the validation pipeline monitors it in the background, so the coordinator does not poll or wait for merge.
- `BU-0180`: An `auto-fix` finding in Sergeant's validation-only workflow is never authorized as an in-run fix; it is instead routed to separate owning-repository task tracker remediation, and a retained gate is never edited, aborted, or restarted to bypass the finding.
- `BU-0310`: The worker never approves a validation gate, routes a finding, or remediates validation output from its own pane; the Sergeant coordinator owns every validation pipeline gate and finding, and the worker resumes only through an explicit Sergeant response tied to that owning work.
- `BU-1226`: When the run output contains a `gate:` object, the pipeline is waiting on the agent; the agent reads its `findings` table, where each finding has an `id`, `severity`, `file`, `description`, and an `action` classifying it.
- `BU-1227`: A finding classified `auto-fix` is mechanical and low-risk, so the agent may authorize the fix on its own judgment.
- `BU-1228`: A finding classified `no-op` is informational only; there is nothing to do about it.
- `BU-1229`: A finding classified `ask-user` challenges the user's deliberate intent or touches product behavior, so it is a decision only the user can make.
- `BU-1230`: Review auto-fix is disabled by default (`auto_fix.review: 0`; a repo or global `auto_fix.review > 0` override re-enables it), so blocking and ask-user review findings park for the agent's/user's decision rather than being silently self-fixed, while other steps such as test and lint may still auto-fix within the pipeline and re-run before ever gating.
- `BU-1231`: The agent may respond the validation pipeline to accept a gated step as-is and continue.
- `BU-1232`: The agent may respond the validation pipeline to have the pipeline fix specific findings and continue.
- `BU-1233`: The agent may respond the validation pipeline to skip the current step.
- `BU-1234`: While a run is active, the agent must never fix findings by editing the code itself, because the pipeline owns both the findings and the fixes; the agent's job at a gate is to decide and respond, and `--action fix` has the pipeline apply the fix and re-review the result.
- `BU-1235`: For the same reason, while a run is active the agent must not `abort` or `rerun` to go fix a finding itself, even a real bug in its own code, because that discards the pipeline's in-flight work and forces a full re-validation; `abort` and `rerun` are for between runs only, never to circumvent a gate.
- `BU-1236`: Each `respond` blocks until the next `gate:`, `checks-passed` decision point, or final outcome.
- `BU-1237`: `--add-finding '<json>'` (with `--action fix`) folds a finding the agent spotted itself, that the pipeline did not surface, into the fix round.
- `BU-1238`: `--step <name>` responds to a specific step instead of the one currently awaiting approval; it is rarely needed and omitted to answer the active gate.
- `BU-1239`: `checks-passed` means the change is validated and CI is green but the PR is not merged yet; the agent is done driving the pipeline and must not wait for the merge, instead telling the user the PR is ready for review and merge (the PR link is in the `help` line), while the validation pipeline keeps monitoring the PR in the background until it is merged, closed, or its configured idle timeout elapses.
- `BU-1240`: `passed` means the changes cleared the gate and the PR was merged or closed.
- `BU-1250`: The CI step deliberately keeps watching the PR after checks pass, so the pipeline-automation tool returns `checks-passed` the moment checks are green rather than blocking on the human merge, and the agent must never poll or re-run waiting for the merge.
- `BU-1253`: On a successful outcome, the agent closes the loop with the user by summarizing what the pipeline validated and found in a concise, easily readable format, and if the output includes a `fixes` table, explicitly acknowledges and lists each fix the pipeline made that the original change missed.
- `BU-1254`: A gate whose findings are all `auto-fix` or `no-op` is safe for the agent to drive on its own judgment, but a finding marked `ask-user` must not be approved, fixed, or skipped by the agent on its own; instead the agent stops and brings it to the user before responding.
- `BU-1255`: The agent relays each `ask-user` finding to the user exactly as the pipeline wrote it — its `id`, `file`, and full `description` verbatim — without paraphrasing, summarizing away detail, or pre-judging the answer.
- `BU-1256`: The agent asks the user how they want to proceed, then translates their decision into the matching `respond` call: `--action fix` (with their guidance passed through `--instructions`), `--action approve`, or `--action skip`.
- `BU-1257`: The one exception to escalating `ask-user` findings is `--yes`: the user's standing consent to drive every gate unattended, under which the agent resolves `ask-user` findings automatically instead of stopping to ask.
- `BU-1258`: With clear consent to drive automatically, the agent passes `--yes` to the pipeline-automation tool, which treats every actionable finding (`auto-fix` and `ask-user` alike) as consent to fix, selects every current finding for one fix round, accepts the resulting fix review, and approves gates with only `no-op` findings; it is used only when the user has asked to drive the whole run without checking back.

