# 90-handover-log: handover log

## Inputs

| File | Layer | Why |
|---|---|---|
| ../80-close-out/output/README.md | L4 | upstream artifact produced by `80-close-out` |

## Purpose

Every ownership transfer is appended to an owner-only log; release tokens are single-use.

Trigger (workflow-level): Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## What must become true here (durable outcome)

Every ownership transfer is appended to an owner-only log; release tokens are single-use.

## Behavior contract

- **Every ownership transfer is durably appended to an owner-only handover log recording timestamp, reason, repository, prior and new pane, and both identity tuples, and a release token is consumed by the claim that uses it so it can never be replayed by a third pane later.**
  (trigger: an ownership claim or release occurs; outcome: every transfer is durably auditable and a release can never be reused by an unintended third party)
  — `BU-P8-089`, `reference/sergeant-upstream/docs/using-sergeant.md` (L376-379)
- **Coordinator-owned validation must distinguish its own diagnostics from a worker's, and must verify a handed-over coordinator pane's identity before proceeding, so validation ownership (who is allowed to run/approve no-mistakes) is never ambiguous between a worker pane and the coordinator.**
  (trigger: coordinator-owned no-mistakes validation is launched, possibly after a pane handover; outcome: validation authority is unambiguous and independently verified, matching the worker-brief's own rule that the coordinator owns every no-mistakes gate and a worker pane may never approve or route one)
  — `BU-P7-104`, `reference/sergeant-upstream/tests/sgt-validate-test.sh` (line 1180)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P8-089`, `BU-P7-104` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
