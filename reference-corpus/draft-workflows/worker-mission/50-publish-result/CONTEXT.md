# 50-publish-result: publish result

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-escalate-or-continue/output/README.md | L4 | upstream artifact produced by `40-escalate-or-continue` |

## Purpose

Handoff evidence recorded from the verified work surface; readiness bounded and reported rather than hanging.

Trigger (workflow-level): A worker starts against a rendered brief.

## What must become true here (durable outcome)

Handoff evidence recorded from the verified work surface; readiness bounded and reported rather than hanging.

## Behavior contract

- **sgt-td-memory must record handoff evidence only from a verified worktree, and every git field it stores (branch, HEAD, etc.) must resolve from that specific worktree rather than from the supervisor's own current working directory — proven with two real linked worktrees on different branches/commits, not simulated.**
  (trigger: sgt-td-memory records recovery evidence for a worker; outcome: recorded recovery evidence (branch, commit, etc.) always describes the worker's actual worktree, never an ambient/wrong working directory, even under multi-worktree git setups)
  — `BU-P7-066`, `reference/sergeant-upstream/tests/sgt-td-memory-worktree-test.sh` (lines 1-18)
- **The interactive worker's wait for harness readiness must be bounded and its outcome reported — a harness that never renders must be caught and reported, not hang forever — and separately, a harness that reaches its pane without ever acknowledging the notification must NOT be misrecorded as orphaned.**
  (trigger: a launched harness process may never become ready for input; outcome: the worker never spins forever waiting for readiness, and its eventual diagnosis distinguishes 'harness never became ready' from 'harness ready but never acknowledged' rather than conflating both into a generic orphaned status)
  — `BU-P7-110`, `reference/sergeant-upstream/tests/sgt-worker-readiness-test.sh` (lines 1-9)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P7-110` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
