# 70-launch-and-record: launch and record

## Inputs

| File | Layer | Why |
|---|---|---|
| ../60-render-brief/output/README.md | L4 | upstream artifact produced by `60-render-brief` |

## Purpose

Launch evidence is written `intended` then promoted to `confirmed` only on observed readiness; every per-repo failure records an orphaned status with a diagnostic before the loop aborts.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

Launch evidence is written `intended` then promoted to `confirmed` only on observed readiness; every per-repo failure records an orphaned status with a diagnostic before the loop aborts.

## Behavior contract

- **Launch evidence distinguishes intent from proof by a two-state field: it is written as 'intended' before the harness is even invoked, and only promoted to 'confirmed' once the harness is observably ready, so a harness that rejects the pin and exits immediately never leaves evidence claiming the model actually ran.**
  (trigger: launch evidence is written both before and after the harness actually starts; outcome: launch evidence can never overclaim: an aborted launch is never mistaken for a confirmed run)
  — `BU-P6-110`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L86-90)
- **Every repo's identity-switching, worktree acquisition, brief writing, and worker launch is validated in order such that any failure records an orphaned status with a specific diagnostic and a handoff before the dispatch loop aborts, rather than leaving a repo's fleet state silently incomplete.**
  (trigger: any step of launching one repo's worker fails partway through; outcome: a partial dispatch failure always leaves an explicitly diagnosable orphaned state with a handoff, never a silently stuck or ambiguous fleet record)
  — `BU-P6-126`, `reference/sergeant-upstream/bin/sgt-dispatch` (L924-928, L939-943, L949-953, L959-963)
- **The interactive worker must pass its pinned provider/model tuple to the harness process it launches, and the durable launch_record it writes must be verified against the harness's own independently observed argv/environment — never trusted from the worker's internal bookkeeping alone — so recorded launch evidence is provably what actually ran.**
  (trigger: the interactive worker launches its harness process with a pinned provider/model tuple; outcome: recorded launch evidence is independently verifiable against the actual process's own observed argv/environment, not merely self-reported by the launching code)
  — `BU-P7-111`, `reference/sergeant-upstream/tests/sgt-worker-model-tuple-test.sh` (lines 6-9)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
